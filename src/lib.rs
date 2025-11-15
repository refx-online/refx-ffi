use refx_pp::{
    Beatmap,
    model::mode::GameMode,
    any::PerformanceAttributes,
};
use interoptopus::{
    extra_type, ffi_function, ffi_type, function, patterns::option::FFIOption, Inventory,
    InventoryBuilder,
};
use std::ffi::CStr;
use std::os::raw::c_char;
use std::path::Path;

#[ffi_type]
#[repr(C)]
#[derive(Clone, Default, PartialEq)]
pub struct CalculatePerformanceResult {
    pub pp: f64,
    pub stars: f64,
}

impl std::fmt::Display for CalculatePerformanceResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("CalculateResult");
        s.field("pp", &self.pp).field("stars", &self.stars);

        s.finish()
    }
}

impl CalculatePerformanceResult {
    fn from_attributes(attributes: PerformanceAttributes) -> Self {
        Self {
            pp: attributes.pp(),
            stars: attributes.stars(),
        }
    }
}

#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn calculate_score(
    beatmap_path: *const c_char,
    mode: u32,
    mods: *const c_char,
    max_combo: u32,
    accuracy: f64,
    miss_count: u32,
    passed_objects: FFIOption<u32>,
    #[allow(unused_variables)]
    lazer: bool, // unused
    score: u32,
) -> CalculatePerformanceResult {
    let path_str = CStr::from_ptr(beatmap_path).to_str().unwrap();
    let beatmap = Beatmap::from_path(Path::new(path_str)).unwrap();
    
    // NOTE: im not gonna update the client cuz my pc is VERY slow to even open the project
    //       so im gonna use this very *CURSED* str to u32 because
    //       the client passes (uint)mods.to_string() to the ffi
    //       https://stackoverflow.com/questions/66582380/pass-string-from-c-sharp-to-rust-using-ffi
    let mods: u32 = { 
        CStr::from_ptr(mods)
            .to_str()
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0)
    };
    
    let mode = match mode {
        0 => GameMode::Osu,
        1 => GameMode::Taiko,
        2 => GameMode::Catch,
        3 => GameMode::Mania,
        _ => panic!("Invalid mode"),
    };
    
    let mut calculator = beatmap
        .performance()
        .mods(mods)
        .combo(max_combo)
        .misses(miss_count)
        .accuracy(accuracy)
        .legacy_total_score(i64::from(score))
        .mode_or_ignore(mode);

    if let Some(passed_objects) = passed_objects.into_option() {
        calculator = calculator.passed_objects(passed_objects);
    }

    let rosu_result = calculator.calculate();
    CalculatePerformanceResult::from_attributes(rosu_result)
}

pub fn my_inventory() -> Inventory {
    InventoryBuilder::new()
        .register(extra_type!(CalculatePerformanceResult))
        .register(function!(calculate_score))
        .inventory()
}
