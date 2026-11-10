// runtime.rs
use std::cell::RefCell;

pub const ID: &str = "auto_eat_baritone";
const HUNGER_THRESHOLD: i32 = 8;
const CHECK_INTERVAL_TICKS: u64 = 20; // once per second
const MUSHROOM_TARGET: u32 = 6;

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

#[derive(Default)]
struct State {
    ticks: u64,
    gathering: bool,
    saved_goal: Option<String>, // e.g. "#goto 100 128 200"
    cooldown: u32,
}

pub fn on_tick(game: &mut crate::game::Game<'_>) {
    if !game.native_on(ID) || !game.native_playing() {
        return;
    }

    STATE.with(|s| {
        let mut st = s.borrow_mut();
        st.ticks += 1;
        if st.ticks % CHECK_INTERVAL_TICKS != 0 {
            return;
        }
        if st.cooldown > 0 {
            st.cooldown -= 1;
            return;
        }

        // Hunger check (use native_player or food helpers if available)
        // For simplicity we re-check inventory every second
        try_eat_or_gather(game, &mut st);
    });
}

fn try_eat_or_gather(game: &mut crate::game::Game<'_>, st: &mut State) {
    // 1. Prefer any food already in inventory
    if let Some(food) = find_food(game) {
        eat_item(game, &food);
        st.cooldown = 40; // ~2s after eating
        return;
    }

    // 2. Try mushroom stew
    let red = game.native_count_items("minecraft:red_mushroom");
    let brown = game.native_count_items("minecraft:brown_mushroom");
    let bowls = game.native_count_items("minecraft:bowl");

    if red >= 1 && brown >= 1 && bowls >= 1 {
        // Simplified: take ingredients and give stew
        // Real crafting needs more hooks or native_place_recipe if available
        game.native_take_items("minecraft:red_mushroom", 1);
        game.native_take_items("minecraft:brown_mushroom", 1);
        game.native_take_items("minecraft:bowl", 1);
        game.native_give_items("minecraft:mushroom_stew", 1);
        eat_item(game, "minecraft:mushroom_stew");
        st.cooldown = 40;
        return;
    }

    // 3. Mushrooms low → tell Baritone to gather
    if red + brown < MUSHROOM_TARGET && !st.gathering {
        st.gathering = true;
        // Save current Baritone goal if you can detect it; otherwise just send mine commands
        send_baritone(game, "#mine red_mushroom");
        send_baritone(game, "#mine brown_mushroom");
        // After gathering you would resume with the saved goal
    }
}

fn find_food(game: &mut crate::game::Game<'_>) -> Option<String> {
    // Expand this list as needed
    const FOODS: &[&str] = &[
        "minecraft:cooked_beef",
        "minecraft:cooked_porkchop",
        "minecraft:cooked_chicken",
        "minecraft:bread",
        "minecraft:golden_carrot",
        "minecraft:mushroom_stew",
        "minecraft:beetroot_soup",
        // ...
    ];
    for id in FOODS {
        if game.native_count_items(id) > 0 {
            return Some(id.to_string());
        }
    }
    None
}

fn eat_item(game: &mut crate::game::Game<'_>, item: &str) {
    // Take one and (ideally) start using it.
    // Full “use item” automation may need native_use_item or packet sending.
    let _ = game.native_take_items(item, 1);
    // Optional: game.native_send_chat("#use"); or whatever Baritone/your setup supports
}

fn send_baritone(game: &mut crate::game::Game<'_>, cmd: &str) {
    // native_send_chat sends the line as if the player typed it
    let _ = game.native_send_chat(cmd);
}
