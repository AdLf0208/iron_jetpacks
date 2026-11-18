//! Iron Jetpacks — the one `impl Hooks` at the crate root.
//!
//! The pure flight model, the command parser and the guest-flag decoder live in
//! [`runtime`], a plain module with no `mod_api` types, so `cargo test` exercises
//! the numbers without the game. This file is the glue: it reads the game through
//! typed hosts, calls the runtime, and writes the result back.
//!
//! The tank is on the **item** (the chest stack's `minecraft:custom_data`
//! `horizon_energy`, the same place the retired bundle's item-energy helper
//! used), and the engine is the mod's own `ironjetpacks:engine` component.
use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard};

use mod_api::{
    self, ClientHost, ClientTickPreEvent, Command, CommandArgumentValue, CommandInvocation,
    CommandSource, Component, ComponentTypesEvent, Compound, EnergyHandler, EquipmentSlot, Flow,
    GameMode, Hooks, HostSide, ITEM_ENERGY, Identifier, ItemPlace, ItemStack, PlayerInputEvent,
    PlayerRef, PlayerTickPostEvent, RegisterCommandsEvent, Registrar, RegistrationContext,
    RunModCommandEvent, ServerHost, Tag, Text, Transaction,
};
use runtime::{Action, Flags, IRON, Input, State, input_from_flags, parse, step};

/// The single 26.1 jetpack item; the port fixes its tier to iron.
const ITEM: &str = "ironjetpacks:jetpack";
/// The mod's own item component: the engine switch (upstream `JETPACK_ENGINE`).
const ENGINE: &str = "ironjetpacks:engine";
/// Where the item's energy lives, as the retired bundle's item-energy buffer kept it.
const CUSTOM_DATA: &str = "minecraft:custom_data";
const ENERGY_KEY: &str = "horizon_energy";

#[derive(Default)]
struct Mod;

/// What one process remembers between hooks: the window's jump key and the
/// flags a guest last sent (`player_input`). Both are keyed by player UUID.
#[derive(Default)]
struct Store {
    jump_held: BTreeMap<u128, bool>,
    guest_input: BTreeMap<u128, Flags>,
}

static STORE: Mutex<Option<Store>> = Mutex::new(None);

fn store() -> MutexGuard<'static, Option<Store>> {
    STORE.lock().expect("ironjetpacks store lock poisoned")
}

/// `minecraft:custom_data`, read and written as the compound it is.
struct CustomData;

impl Component for CustomData {
    type Value = Compound;

    fn id(&self) -> &str {
        CUSTOM_DATA
    }

    fn decode(&self, value: &Tag) -> Option<Compound> {
        match value {
            Tag::Compound(data) => Some(data.clone()),
            _ => None,
        }
    }

    fn encode(&self, value: Compound) -> Tag {
        Tag::Compound(value)
    }
}

/// The mod's `ironjetpacks:engine` switch, kept as a byte (`1b` on, `0b` off).
struct Engine;

impl Component for Engine {
    type Value = bool;

    fn id(&self) -> &str {
        ENGINE
    }

    fn decode(&self, value: &Tag) -> Option<bool> {
        match value {
            Tag::Byte(on) => Some(*on != 0),
            _ => None,
        }
    }

    fn encode(&self, value: bool) -> Tag {
        Tag::Byte(i8::from(value))
    }
}

/// The energy one jetpack stack holds; a missing or malformed key is empty.
fn energy(stack: &ItemStack) -> u32 {
    let Some(data) = stack.get(&CustomData) else {
        return 0;
    };
    let data = Tag::Compound(data);
    match data.get(ENERGY_KEY) {
        Some(Tag::Int(value)) => u32::try_from(*value).unwrap_or(0),
        Some(Tag::Long(value)) => u32::try_from(*value).unwrap_or(0),
        _ => 0,
    }
}

/// Writes the stack's energy; zero removes the key, as the retired tank did.
fn with_energy(stack: &mut ItemStack, amount: u32) {
    let mut kept: Compound = stack
        .get(&CustomData)
        .unwrap_or_default()
        .into_iter()
        .filter(|(key, _)| key.to_string_lossy() != ENERGY_KEY)
        .collect();
    if amount > 0 {
        let amount = i32::try_from(amount).unwrap_or(i32::MAX);
        kept.insert(ENERGY_KEY.into(), Tag::Int(amount));
    }
    if kept.is_empty() {
        stack.remove(&CustomData);
    } else {
        let _ = stack.set(&CustomData, kept);
    }
}

fn engine_on(stack: &ItemStack) -> bool {
    stack.get(&Engine).unwrap_or(false)
}

/// Whether the stack is this port's jetpack item.
fn is_jetpack(host: &dyn ServerHost, stack: &ItemStack) -> bool {
    Identifier::parse_namespaced(ITEM).and_then(|id| host.item_key(&id)) == Some(stack.item())
}

fn with_engine(stack: &mut ItemStack, on: bool) {
    let _ = stack.set(&Engine, on);
}

/// `ITEM_ENERGY` for the jetpack: the iron tier's tank, read and written in the
/// stack's own custom data through the place the game asked about.
struct JetpackTank {
    place: ItemPlace,
}

impl EnergyHandler for JetpackTank {
    fn amount(&self, host: &mut dyn ServerHost) -> u32 {
        self.place.stack(host).map_or(0, |stack| energy(&stack))
    }

    fn capacity(&self, _host: &mut dyn ServerHost) -> u32 {
        IRON.capacity
    }

    fn insert(&self, host: &mut dyn ServerHost, tx: &Transaction, amount: u32) -> u32 {
        let Some(stack) = self.place.stack(host) else {
            return 0;
        };
        let before = energy(&stack);
        let moved = amount.min(IRON.capacity.saturating_sub(before));
        if moved == 0
            || !self
                .place
                .edit(host, tx, |stack| with_energy(stack, before + moved))
        {
            return 0;
        }
        moved
    }

    fn extract(&self, host: &mut dyn ServerHost, tx: &Transaction, amount: u32) -> u32 {
        let Some(stack) = self.place.stack(host) else {
            return 0;
        };
        let before = energy(&stack);
        let moved = amount.min(before);
        if moved == 0
            || !self
                .place
                .edit(host, tx, |stack| with_energy(stack, before - moved))
        {
            return 0;
        }
        moved
    }
}

/// The input one player's tick uses: a guest's own flags, else the window's
/// jump key for the local player.
fn input_for(host: &dyn ServerHost, player: PlayerRef) -> Input {
    let entity = player.entity();
    let sprinting = entity.is_sprinting(host).unwrap_or(false);
    let on_ground = entity.on_ground(host).unwrap_or(false);
    let in_water = entity.is_in_water(host).unwrap_or(false);
    let mut guard = store();
    let store = guard.get_or_insert_with(Store::default);
    match store.guest_input.get(&player.uuid()).copied() {
        Some(flags) => Input {
            up: flags.up,
            down: flags.down,
            sprint: flags.sprint || sprinting,
            on_ground,
            in_water,
        },
        None => Input {
            up: store
                .jump_held
                .get(&player.uuid())
                .copied()
                .unwrap_or(false),
            down: false,
            sprint: sprinting,
            on_ground,
            in_water,
        },
    }
}

/// One player's server tick end: thrust while the iron jetpack is worn in the
/// chest slot, its engine is on and the ascend key is held.
fn fly(host: &mut dyn ServerHost, player: PlayerRef) {
    let Some(living) = host.living(player.entity()) else {
        return;
    };
    let Some(stack) = living.equipment(host, EquipmentSlot::Chest) else {
        return;
    };
    if !is_jetpack(&*host, &stack) {
        return;
    }
    let Some(mode) = player.game_mode(host) else {
        return;
    };
    if mode == GameMode::Spectator {
        return;
    }
    let velocity = player.entity().velocity(host).unwrap_or([0.0; 3]);
    let before = State {
        engine_on: engine_on(&stack),
        hover: false,
        creative: mode == GameMode::Creative,
        throttle: 1.0,
        fuel: energy(&stack),
    };
    let stepped = step(&IRON, before, input_for(&*host, player), velocity[1]);
    if stepped.spent > 0 {
        let mut one = stack;
        with_energy(&mut one, stepped.fuel);
        let _ = living.set_equipment(host, EquipmentSlot::Chest, Some(one));
    }
    let Some(target) = stepped.velocity_y else {
        return;
    };
    // `entity_set_velocity` refuses a player (its client moves it); `Entity.push`
    // does not, and adding the difference lands exactly at the runtime's target
    // while leaving the other axes alone. The server's next tick sends the change
    // to the player's client as `SetEntityMotion`.
    let delta = target - velocity[1];
    let _ = player.entity().push(host, [0.0, delta, 0.0]);
    // `PlayerTickHandler.java:91`: fall protection on every thrust tick.
    player.entity().reset_fall_distance(host);
}

/// The command line the runtime's parser understands, rebuilt from the parsed
/// invocation (literals plus resolved arguments).
fn command_line(invocation: &CommandInvocation) -> String {
    let mut parts = invocation.literals.clone();
    for argument in &invocation.arguments {
        match &argument.value {
            CommandArgumentValue::Integer(value) => parts.push(value.to_string()),
            CommandArgumentValue::Boolean(value) => {
                parts.push(if *value { "on" } else { "off" }.to_owned());
            }
            CommandArgumentValue::String(value) => parts.push(value.clone()),
            _ => {}
        }
    }
    parts.join(" ")
}

fn feedback(host: &mut dyn ServerHost, line: String) {
    let _ = host.commands().feedback(Text::literal(line));
}

/// The worn jetpack the command acts on, or the feedback saying there is none.
fn worn(host: &mut dyn ServerHost, player: PlayerRef) -> Option<(mod_api::LivingRef, ItemStack)> {
    let living = host.living(player.entity())?;
    let stack = living.equipment(host, EquipmentSlot::Chest)?;
    is_jetpack(&*host, &stack).then_some((living, stack))
}

fn perform(host: &mut dyn ServerHost, source: &CommandSource, action: Action) {
    let player = match source {
        CommandSource::Player { player, .. } => *player,
        _ => {
            feedback(
                host,
                "Iron Jetpacks: wear the jetpack in the chest slot".to_owned(),
            );
            return;
        }
    };
    let Some((living, stack)) = worn(host, player) else {
        feedback(
            host,
            "Iron Jetpacks: wear the jetpack in the chest slot".to_owned(),
        );
        return;
    };
    match action {
        Action::Unknown => {}
        Action::Bad => feedback(
            host,
            "Usage: /ironjetpacks fuel [amount] | /ironjetpacks engine [on|off]".to_owned(),
        ),
        Action::Query => feedback(
            host,
            format!(
                "Iron Jetpacks: {} / {} FE, engine {}",
                energy(&stack),
                IRON.capacity,
                if engine_on(&stack) { "on" } else { "off" }
            ),
        ),
        Action::Set(amount) => {
            let mut one = stack;
            with_energy(&mut one, amount);
            let after = energy(&one);
            let _ = living.set_equipment(host, EquipmentSlot::Chest, Some(one));
            feedback(
                host,
                format!("Iron Jetpacks: fuel set to {after} / {}", IRON.capacity),
            );
        }
        Action::ToggleEngine | Action::SetEngine(_) => {
            let on = match action {
                Action::SetEngine(on) => on,
                _ => !engine_on(&stack),
            };
            let mut one = stack;
            with_engine(&mut one, on);
            let _ = living.set_equipment(host, EquipmentSlot::Chest, Some(one));
            feedback(
                host,
                format!("Iron Jetpacks: engine {}", if on { "on" } else { "off" }),
            );
        }
    }
}

#[mod_api::horizon_mod]
impl Hooks for Mod {
    fn register(&self, r: &mut dyn Registrar, _cx: &RegistrationContext) {
        r.item_capability(
            &ITEM_ENERGY,
            ITEM,
            |_host: &mut dyn ServerHost, place: &ItemPlace| {
                Some(Box::new(JetpackTank {
                    place: place.clone(),
                }) as Box<dyn EnergyHandler>)
            },
        );
    }

    fn component_types(&self, _cx: &RegistrationContext, event: &mut ComponentTypesEvent<'_>) {
        event.types.push(ENGINE.to_owned());
    }

    fn client_tick_pre(&self, host: &mut dyn ClientHost, _event: &mut ClientTickPreEvent) {
        let Some(player) = host.local_player() else {
            return;
        };
        let down = host
            .mapping("key.jump")
            .ok()
            .flatten()
            .is_some_and(|mapping| mapping.down);
        let mut guard = store();
        guard
            .get_or_insert_with(Store::default)
            .jump_held
            .insert(player.uuid(), down);
    }

    fn player_input(&self, _host: &mut dyn ServerHost, event: &mut PlayerInputEvent<'_>) {
        let flags = input_from_flags(event.flags);
        let mut guard = store();
        guard
            .get_or_insert_with(Store::default)
            .guest_input
            .insert(event.player, flags);
    }

    fn player_tick_post(&self, host: HostSide, event: &mut PlayerTickPostEvent) {
        if let HostSide::Server(host) = host {
            fly(host, event.player);
        }
    }

    fn register_commands(&self, _host: &mut dyn ServerHost, event: &mut RegisterCommandsEvent<'_>) {
        event.commands.push(Command::op("ironjetpacks fuel", 2));
        event
            .commands
            .push(Command::op("ironjetpacks fuel <amount:integer>", 2));
        event.commands.push(Command::new("ironjetpacks engine"));
        event
            .commands
            .push(Command::new("ironjetpacks engine <on:boolean>"));
    }

    fn run_mod_command(
        &self,
        host: &mut dyn ServerHost,
        event: &mut RunModCommandEvent<'_>,
    ) -> Flow {
        let action = parse(&command_line(event.command));
        if action == Action::Unknown {
            return Flow::Continue;
        }
        perform(host, event.source, action);
        Flow::Cancel
    }
}

pub mod runtime;
