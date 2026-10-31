//! Motion Music's curated palettes, available without files or network access.
use super::Palette;
use egui::Color32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MotionTheme {
    Amethyst,
    PurpleRain,
    Ultraviolet,
    VioletHour,
    Nebula,
    GrapeSoda,
    CyberOrchid,
    MidnightLilac,
    PlumVelvet,
    Blacklight,
    Synthwave,
    CosmicDusk,
    Crimson,
    CherryBomb,
    Ruby,
    Scarlet,
    BloodMoon,
    Ember,
    RoseNoir,
    Inferno,
    Burgundy,
    HotLava,
    Arctic,
    DeepOcean,
    ElectricBlue,
    NorthernLights,
    Emerald,
    ToxicMint,
    LiquidGold,
    Graphite,
    LavenderMilk,
    LilacCloud,
    OrchidGlass,
    RoseQuartz,
    CherryBlossom,
    PeachFuzz,
    Glacier,
    Seafoam,
    LemonSorbet,
    Paper,
    VelvetViolet,
    OrchidNight,
    ElectricPlum,
    Iris,
    Dragonfruit,
    ObsidianViolet,
    Vermilion,
    RedVelvet,
    Carmine,
    CoralFlare,
    Merlot,
    Pomegranate,
    JadeTemple,
    Malachite,
    TidalTeal,
    Sapphire,
    Tangerine,
    AmberRoom,
    Citron,
    GlacialBlue,
    Wisteria,
    BlushSilk,
    MintTea,
    Apricot,
    PrismNight,
    VelvetSunset,
    AuroraBorealis,
    MidnightLagoon,
    CrimsonNebula,
    NeonHorizon,
    PurpleHaze,
    CherryAurora,
    DeepSpace,
    MoltenGold,
    EmeraldDream,
    OceanDrive,
    CandyCloud,
    DawnGlow,
    Iridescent,
    Rosewater,
    CobaltInk,
    PetrolBlue,
    PacificNight,
    ForestCanopy,
    PineNeedle,
    OliveGrove,
    BronzeAge,
    CopperWire,
    DesertNight,
    Espresso,
    SlateSteel,
    CarbonMono,
    Bubblegum,
    Peacock,
    Marigold,
    Persimmon,
    PowderBlue,
    Pistachio,
    Buttercream,
    Sandstone,
    OatMilk,
    PinkLemonade,
    LagoonMist,
    SilverLining,
    SolarFlare,
    BorealSky,
    SapphireRose,
    CopperTwilight,
    TropicalNight,
    AmberTide,
    ForestEmber,
    PinkVoltage,
    BlueHour,
    CitrusNoir,
    CobaltEmber,
    OpalNight,
    DesertMirage,
    MoonlitSteel,
    PeacockFeather,
    RoyalSpectrum,
    SorbetSky,
    SeaglassDawn,
    LemonPetal,
    PeachLagoon,
    PearlPrism,
    SkyMeadow,
    CoralMist,
    HoneyLavender,
    PoppyRed,
    Mandarin,
    MangoMilk,
    Sunflower,
    Daffodil,
    LimeWire,
    MatchaLatte,
    RacingGreen,
    Viridian,
    AquaMarine,
    CyanCircuit,
    IceMint,
    RoyalNavy,
    Cornflower,
    FuchsiaPulse,
    SakuraSilk,
    Terracotta,
    ToffeeCream,
    TrueBlack,
    SilverMist,
    PureWhite,
    AshStone,
    LavenderInk,
    LilacPetal,
    RubySunrise,
    MangoWave,
    LemonLime,
    JungleCurrent,
    CyanAurora,
    IndigoSpectrum,
    FuchsiaLagoon,
    BronzeOcean,
    NoirPrism,
    CopperRose,
    CitrusClouds,
    AquaBloom,
    BluePorcelain,
    RoseGold,
    PistachioPeach,
    ArcticPearl,
    RubySilk,
    ScarletEmber,
    GarnetGlow,
    CherryLacquer,
    Rosefire,
    CrimsonTide,
    VelvetRuby,
    BloodOrange,
    StrawberryCream,
    CherrySorbet,
    RoseSatin,
    CoralVelvet,
}

impl MotionTheme {
    /// A reserved in-memory palette identity; this is never a file path.
    pub fn filename(self) -> &'static str {
        match self {
            Self::Amethyst => "motion-theme:amethyst",
            Self::PurpleRain => "motion-theme:purple-rain",
            Self::Ultraviolet => "motion-theme:ultraviolet",
            Self::VioletHour => "motion-theme:violet-hour",
            Self::Nebula => "motion-theme:nebula",
            Self::GrapeSoda => "motion-theme:grape-soda",
            Self::CyberOrchid => "motion-theme:cyber-orchid",
            Self::MidnightLilac => "motion-theme:midnight-lilac",
            Self::PlumVelvet => "motion-theme:plum-velvet",
            Self::Blacklight => "motion-theme:blacklight",
            Self::Synthwave => "motion-theme:synthwave",
            Self::CosmicDusk => "motion-theme:cosmic-dusk",
            Self::Crimson => "motion-theme:crimson",
            Self::CherryBomb => "motion-theme:cherry-bomb",
            Self::Ruby => "motion-theme:ruby",
            Self::Scarlet => "motion-theme:scarlet",
            Self::BloodMoon => "motion-theme:blood-moon",
            Self::Ember => "motion-theme:ember",
            Self::RoseNoir => "motion-theme:rose-noir",
            Self::Inferno => "motion-theme:inferno",
            Self::Burgundy => "motion-theme:burgundy",
            Self::HotLava => "motion-theme:hot-lava",
            Self::Arctic => "motion-theme:arctic",
            Self::DeepOcean => "motion-theme:deep-ocean",
            Self::ElectricBlue => "motion-theme:electric-blue",
            Self::NorthernLights => "motion-theme:northern-lights",
            Self::Emerald => "motion-theme:emerald",
            Self::ToxicMint => "motion-theme:toxic-mint",
            Self::LiquidGold => "motion-theme:liquid-gold",
            Self::Graphite => "motion-theme:graphite",
            Self::LavenderMilk => "motion-theme:lavender-milk",
            Self::LilacCloud => "motion-theme:lilac-cloud",
            Self::OrchidGlass => "motion-theme:orchid-glass",
            Self::RoseQuartz => "motion-theme:rose-quartz",
            Self::CherryBlossom => "motion-theme:cherry-blossom",
            Self::PeachFuzz => "motion-theme:peach-fuzz",
            Self::Glacier => "motion-theme:glacier",
            Self::Seafoam => "motion-theme:seafoam",
            Self::LemonSorbet => "motion-theme:lemon-sorbet",
            Self::Paper => "motion-theme:paper",
            Self::VelvetViolet => "motion-theme:velvet-violet",
            Self::OrchidNight => "motion-theme:orchid-night",
            Self::ElectricPlum => "motion-theme:electric-plum",
            Self::Iris => "motion-theme:iris",
            Self::Dragonfruit => "motion-theme:dragonfruit",
            Self::ObsidianViolet => "motion-theme:obsidian-violet",
            Self::Vermilion => "motion-theme:vermilion",
            Self::RedVelvet => "motion-theme:red-velvet",
            Self::Carmine => "motion-theme:carmine",
            Self::CoralFlare => "motion-theme:coral-flare",
            Self::Merlot => "motion-theme:merlot",
            Self::Pomegranate => "motion-theme:pomegranate",
            Self::JadeTemple => "motion-theme:jade-temple",
            Self::Malachite => "motion-theme:malachite",
            Self::TidalTeal => "motion-theme:tidal-teal",
            Self::Sapphire => "motion-theme:sapphire",
            Self::Tangerine => "motion-theme:tangerine",
            Self::AmberRoom => "motion-theme:amber-room",
            Self::Citron => "motion-theme:citron",
            Self::GlacialBlue => "motion-theme:glacial-blue",
            Self::Wisteria => "motion-theme:wisteria",
            Self::BlushSilk => "motion-theme:blush-silk",
            Self::MintTea => "motion-theme:mint-tea",
            Self::Apricot => "motion-theme:apricot",
            Self::PrismNight => "motion-theme:prism-night",
            Self::VelvetSunset => "motion-theme:velvet-sunset",
            Self::AuroraBorealis => "motion-theme:aurora-borealis",
            Self::MidnightLagoon => "motion-theme:midnight-lagoon",
            Self::CrimsonNebula => "motion-theme:crimson-nebula",
            Self::NeonHorizon => "motion-theme:neon-horizon",
            Self::PurpleHaze => "motion-theme:purple-haze",
            Self::CherryAurora => "motion-theme:cherry-aurora",
            Self::DeepSpace => "motion-theme:deep-space",
            Self::MoltenGold => "motion-theme:molten-gold",
            Self::EmeraldDream => "motion-theme:emerald-dream",
            Self::OceanDrive => "motion-theme:ocean-drive",
            Self::CandyCloud => "motion-theme:candy-cloud",
            Self::DawnGlow => "motion-theme:dawn-glow",
            Self::Iridescent => "motion-theme:iridescent",
            Self::Rosewater => "motion-theme:rosewater",
            Self::CobaltInk => "motion-theme:cobalt-ink",
            Self::PetrolBlue => "motion-theme:petrol-blue",
            Self::PacificNight => "motion-theme:pacific-night",
            Self::ForestCanopy => "motion-theme:forest-canopy",
            Self::PineNeedle => "motion-theme:pine-needle",
            Self::OliveGrove => "motion-theme:olive-grove",
            Self::BronzeAge => "motion-theme:bronze-age",
            Self::CopperWire => "motion-theme:copper-wire",
            Self::DesertNight => "motion-theme:desert-night",
            Self::Espresso => "motion-theme:espresso",
            Self::SlateSteel => "motion-theme:slate-steel",
            Self::CarbonMono => "motion-theme:carbon-mono",
            Self::Bubblegum => "motion-theme:bubblegum",
            Self::Peacock => "motion-theme:peacock",
            Self::Marigold => "motion-theme:marigold",
            Self::Persimmon => "motion-theme:persimmon",
            Self::PowderBlue => "motion-theme:powder-blue",
            Self::Pistachio => "motion-theme:pistachio",
            Self::Buttercream => "motion-theme:buttercream",
            Self::Sandstone => "motion-theme:sandstone",
            Self::OatMilk => "motion-theme:oat-milk",
            Self::PinkLemonade => "motion-theme:pink-lemonade",
            Self::LagoonMist => "motion-theme:lagoon-mist",
            Self::SilverLining => "motion-theme:silver-lining",
            Self::SolarFlare => "motion-theme:solar-flare",
            Self::BorealSky => "motion-theme:boreal-sky",
            Self::SapphireRose => "motion-theme:sapphire-rose",
            Self::CopperTwilight => "motion-theme:copper-twilight",
            Self::TropicalNight => "motion-theme:tropical-night",
            Self::AmberTide => "motion-theme:amber-tide",
            Self::ForestEmber => "motion-theme:forest-ember",
            Self::PinkVoltage => "motion-theme:pink-voltage",
            Self::BlueHour => "motion-theme:blue-hour",
            Self::CitrusNoir => "motion-theme:citrus-noir",
            Self::CobaltEmber => "motion-theme:cobalt-ember",
            Self::OpalNight => "motion-theme:opal-night",
            Self::DesertMirage => "motion-theme:desert-mirage",
            Self::MoonlitSteel => "motion-theme:moonlit-steel",
            Self::PeacockFeather => "motion-theme:peacock-feather",
            Self::RoyalSpectrum => "motion-theme:royal-spectrum",
            Self::SorbetSky => "motion-theme:sorbet-sky",
            Self::SeaglassDawn => "motion-theme:seaglass-dawn",
            Self::LemonPetal => "motion-theme:lemon-petal",
            Self::PeachLagoon => "motion-theme:peach-lagoon",
            Self::PearlPrism => "motion-theme:pearl-prism",
            Self::SkyMeadow => "motion-theme:sky-meadow",
            Self::CoralMist => "motion-theme:coral-mist",
            Self::HoneyLavender => "motion-theme:honey-lavender",
            Self::PoppyRed => "motion-theme:poppy-red",
            Self::Mandarin => "motion-theme:mandarin",
            Self::MangoMilk => "motion-theme:mango-milk",
            Self::Sunflower => "motion-theme:sunflower",
            Self::Daffodil => "motion-theme:daffodil",
            Self::LimeWire => "motion-theme:lime-wire",
            Self::MatchaLatte => "motion-theme:matcha-latte",
            Self::RacingGreen => "motion-theme:racing-green",
            Self::Viridian => "motion-theme:viridian",
            Self::AquaMarine => "motion-theme:aqua-marine",
            Self::CyanCircuit => "motion-theme:cyan-circuit",
            Self::IceMint => "motion-theme:ice-mint",
            Self::RoyalNavy => "motion-theme:royal-navy",
            Self::Cornflower => "motion-theme:cornflower",
            Self::FuchsiaPulse => "motion-theme:fuchsia-pulse",
            Self::SakuraSilk => "motion-theme:sakura-silk",
            Self::Terracotta => "motion-theme:terracotta",
            Self::ToffeeCream => "motion-theme:toffee-cream",
            Self::TrueBlack => "motion-theme:true-black",
            Self::SilverMist => "motion-theme:silver-mist",
            Self::PureWhite => "motion-theme:pure-white",
            Self::AshStone => "motion-theme:ash-stone",
            Self::LavenderInk => "motion-theme:lavender-ink",
            Self::LilacPetal => "motion-theme:lilac-petal",
            Self::RubySunrise => "motion-theme:ruby-sunrise",
            Self::MangoWave => "motion-theme:mango-wave",
            Self::LemonLime => "motion-theme:lemon-lime",
            Self::JungleCurrent => "motion-theme:jungle-current",
            Self::CyanAurora => "motion-theme:cyan-aurora",
            Self::IndigoSpectrum => "motion-theme:indigo-spectrum",
            Self::FuchsiaLagoon => "motion-theme:fuchsia-lagoon",
            Self::BronzeOcean => "motion-theme:bronze-ocean",
            Self::NoirPrism => "motion-theme:noir-prism",
            Self::CopperRose => "motion-theme:copper-rose",
            Self::CitrusClouds => "motion-theme:citrus-clouds",
            Self::AquaBloom => "motion-theme:aqua-bloom",
            Self::BluePorcelain => "motion-theme:blue-porcelain",
            Self::RoseGold => "motion-theme:rose-gold",
            Self::PistachioPeach => "motion-theme:pistachio-peach",
            Self::ArcticPearl => "motion-theme:arctic-pearl",
            Self::RubySilk => "motion-theme:ruby-silk",
            Self::ScarletEmber => "motion-theme:scarlet-ember",
            Self::GarnetGlow => "motion-theme:garnet-glow",
            Self::CherryLacquer => "motion-theme:cherry-lacquer",
            Self::Rosefire => "motion-theme:rosefire",
            Self::CrimsonTide => "motion-theme:crimson-tide",
            Self::VelvetRuby => "motion-theme:velvet-ruby",
            Self::BloodOrange => "motion-theme:blood-orange",
            Self::StrawberryCream => "motion-theme:strawberry-cream",
            Self::CherrySorbet => "motion-theme:cherry-sorbet",
            Self::RoseSatin => "motion-theme:rose-satin",
            Self::CoralVelvet => "motion-theme:coral-velvet",
        }
    }

    pub fn from_filename(filename: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|preset| preset.filename() == filename)
    }

    pub fn as_custom_theme(self) -> super::CustomTheme {
        super::CustomTheme {
            filename: self.filename().to_owned(),
            palette: self.palette(),
        }
    }

    pub const ALL: [Self; 180] = [
        Self::Amethyst,
        Self::PurpleRain,
        Self::Ultraviolet,
        Self::VioletHour,
        Self::Nebula,
        Self::GrapeSoda,
        Self::CyberOrchid,
        Self::MidnightLilac,
        Self::PlumVelvet,
        Self::Blacklight,
        Self::Synthwave,
        Self::CosmicDusk,
        Self::Crimson,
        Self::CherryBomb,
        Self::Ruby,
        Self::Scarlet,
        Self::BloodMoon,
        Self::Ember,
        Self::RoseNoir,
        Self::Inferno,
        Self::Burgundy,
        Self::HotLava,
        Self::Arctic,
        Self::DeepOcean,
        Self::ElectricBlue,
        Self::NorthernLights,
        Self::Emerald,
        Self::ToxicMint,
        Self::LiquidGold,
        Self::Graphite,
        Self::LavenderMilk,
        Self::LilacCloud,
        Self::OrchidGlass,
        Self::RoseQuartz,
        Self::CherryBlossom,
        Self::PeachFuzz,
        Self::Glacier,
        Self::Seafoam,
        Self::LemonSorbet,
        Self::Paper,
        Self::VelvetViolet,
        Self::OrchidNight,
        Self::ElectricPlum,
        Self::Iris,
        Self::Dragonfruit,
        Self::ObsidianViolet,
        Self::Vermilion,
        Self::RedVelvet,
        Self::Carmine,
        Self::CoralFlare,
        Self::Merlot,
        Self::Pomegranate,
        Self::JadeTemple,
        Self::Malachite,
        Self::TidalTeal,
        Self::Sapphire,
        Self::Tangerine,
        Self::AmberRoom,
        Self::Citron,
        Self::GlacialBlue,
        Self::Wisteria,
        Self::BlushSilk,
        Self::MintTea,
        Self::Apricot,
        Self::PrismNight,
        Self::VelvetSunset,
        Self::AuroraBorealis,
        Self::MidnightLagoon,
        Self::CrimsonNebula,
        Self::NeonHorizon,
        Self::PurpleHaze,
        Self::CherryAurora,
        Self::DeepSpace,
        Self::MoltenGold,
        Self::EmeraldDream,
        Self::OceanDrive,
        Self::CandyCloud,
        Self::DawnGlow,
        Self::Iridescent,
        Self::Rosewater,
        Self::CobaltInk,
        Self::PetrolBlue,
        Self::PacificNight,
        Self::ForestCanopy,
        Self::PineNeedle,
        Self::OliveGrove,
        Self::BronzeAge,
        Self::CopperWire,
        Self::DesertNight,
        Self::Espresso,
        Self::SlateSteel,
        Self::CarbonMono,
        Self::Bubblegum,
        Self::Peacock,
        Self::Marigold,
        Self::Persimmon,
        Self::PowderBlue,
        Self::Pistachio,
        Self::Buttercream,
        Self::Sandstone,
        Self::OatMilk,
        Self::PinkLemonade,
        Self::LagoonMist,
        Self::SilverLining,
        Self::SolarFlare,
        Self::BorealSky,
        Self::SapphireRose,
        Self::CopperTwilight,
        Self::TropicalNight,
        Self::AmberTide,
        Self::ForestEmber,
        Self::PinkVoltage,
        Self::BlueHour,
        Self::CitrusNoir,
        Self::CobaltEmber,
        Self::OpalNight,
        Self::DesertMirage,
        Self::MoonlitSteel,
        Self::PeacockFeather,
        Self::RoyalSpectrum,
        Self::SorbetSky,
        Self::SeaglassDawn,
        Self::LemonPetal,
        Self::PeachLagoon,
        Self::PearlPrism,
        Self::SkyMeadow,
        Self::CoralMist,
        Self::HoneyLavender,
        Self::PoppyRed,
        Self::Mandarin,
        Self::MangoMilk,
        Self::Sunflower,
        Self::Daffodil,
        Self::LimeWire,
        Self::MatchaLatte,
        Self::RacingGreen,
        Self::Viridian,
        Self::AquaMarine,
        Self::CyanCircuit,
        Self::IceMint,
        Self::RoyalNavy,
        Self::Cornflower,
        Self::FuchsiaPulse,
        Self::SakuraSilk,
        Self::Terracotta,
        Self::ToffeeCream,
        Self::TrueBlack,
        Self::SilverMist,
        Self::PureWhite,
        Self::AshStone,
        Self::LavenderInk,
        Self::LilacPetal,
        Self::RubySunrise,
        Self::MangoWave,
        Self::LemonLime,
        Self::JungleCurrent,
        Self::CyanAurora,
        Self::IndigoSpectrum,
        Self::FuchsiaLagoon,
        Self::BronzeOcean,
        Self::NoirPrism,
        Self::CopperRose,
        Self::CitrusClouds,
        Self::AquaBloom,
        Self::BluePorcelain,
        Self::RoseGold,
        Self::PistachioPeach,
        Self::ArcticPearl,
        Self::RubySilk,
        Self::ScarletEmber,
        Self::GarnetGlow,
        Self::CherryLacquer,
        Self::Rosefire,
        Self::CrimsonTide,
        Self::VelvetRuby,
        Self::BloodOrange,
        Self::StrawberryCream,
        Self::CherrySorbet,
        Self::RoseSatin,
        Self::CoralVelvet,
    ];
    pub const FAMILIES: [&str; 12] = [
        "Red",
        "Orange",
        "Yellow",
        "Green",
        "Teal",
        "Cyan",
        "Blue",
        "Purple",
        "Pink",
        "Brown",
        "Monochrome",
        "Gradients",
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Amethyst => "Amethyst",
            Self::PurpleRain => "Purple Rain",
            Self::Ultraviolet => "Ultraviolet",
            Self::VioletHour => "Violet Hour",
            Self::Nebula => "Nebula",
            Self::GrapeSoda => "Grape Soda",
            Self::CyberOrchid => "Cyber Orchid",
            Self::MidnightLilac => "Midnight Lilac",
            Self::PlumVelvet => "Plum Velvet",
            Self::Blacklight => "Blacklight",
            Self::Synthwave => "Synthwave",
            Self::CosmicDusk => "Cosmic Dusk",
            Self::Crimson => "Crimson",
            Self::CherryBomb => "Cherry Bomb",
            Self::Ruby => "Ruby",
            Self::Scarlet => "Scarlet",
            Self::BloodMoon => "Blood Moon",
            Self::Ember => "Ember",
            Self::RoseNoir => "Rose Noir",
            Self::Inferno => "Inferno",
            Self::Burgundy => "Burgundy",
            Self::HotLava => "Hot Lava",
            Self::Arctic => "Arctic",
            Self::DeepOcean => "Deep Ocean",
            Self::ElectricBlue => "Electric Blue",
            Self::NorthernLights => "Northern Lights",
            Self::Emerald => "Emerald",
            Self::ToxicMint => "Toxic Mint",
            Self::LiquidGold => "Liquid Gold",
            Self::Graphite => "Graphite",
            Self::LavenderMilk => "Lavender Milk",
            Self::LilacCloud => "Lilac Cloud",
            Self::OrchidGlass => "Orchid Glass",
            Self::RoseQuartz => "Rose Quartz",
            Self::CherryBlossom => "Cherry Blossom",
            Self::PeachFuzz => "Peach Fuzz",
            Self::Glacier => "Glacier",
            Self::Seafoam => "Seafoam",
            Self::LemonSorbet => "Lemon Sorbet",
            Self::Paper => "Paper",
            Self::VelvetViolet => "Velvet Violet",
            Self::OrchidNight => "Orchid Night",
            Self::ElectricPlum => "Electric Plum",
            Self::Iris => "Iris",
            Self::Dragonfruit => "Dragonfruit",
            Self::ObsidianViolet => "Obsidian Violet",
            Self::Vermilion => "Vermilion",
            Self::RedVelvet => "Red Velvet",
            Self::Carmine => "Carmine",
            Self::CoralFlare => "Coral Flare",
            Self::Merlot => "Merlot",
            Self::Pomegranate => "Pomegranate",
            Self::JadeTemple => "Jade Temple",
            Self::Malachite => "Malachite",
            Self::TidalTeal => "Tidal Teal",
            Self::Sapphire => "Sapphire",
            Self::Tangerine => "Tangerine",
            Self::AmberRoom => "Amber Room",
            Self::Citron => "Citron",
            Self::GlacialBlue => "Glacial Blue",
            Self::Wisteria => "Wisteria",
            Self::BlushSilk => "Blush Silk",
            Self::MintTea => "Mint Tea",
            Self::Apricot => "Apricot",
            Self::PrismNight => "Prism Night",
            Self::VelvetSunset => "Velvet Sunset",
            Self::AuroraBorealis => "Aurora Borealis",
            Self::MidnightLagoon => "Midnight Lagoon",
            Self::CrimsonNebula => "Crimson Nebula",
            Self::NeonHorizon => "Neon Horizon",
            Self::PurpleHaze => "Purple Haze",
            Self::CherryAurora => "Cherry Aurora",
            Self::DeepSpace => "Deep Space",
            Self::MoltenGold => "Molten Gold",
            Self::EmeraldDream => "Emerald Dream",
            Self::OceanDrive => "Ocean Drive",
            Self::CandyCloud => "Candy Cloud",
            Self::DawnGlow => "Dawn Glow",
            Self::Iridescent => "Iridescent",
            Self::Rosewater => "Rosewater",
            Self::CobaltInk => "Cobalt Ink",
            Self::PetrolBlue => "Petrol Blue",
            Self::PacificNight => "Pacific Night",
            Self::ForestCanopy => "Forest Canopy",
            Self::PineNeedle => "Pine Needle",
            Self::OliveGrove => "Olive Grove",
            Self::BronzeAge => "Bronze Age",
            Self::CopperWire => "Copper Wire",
            Self::DesertNight => "Desert Night",
            Self::Espresso => "Espresso",
            Self::SlateSteel => "Slate Steel",
            Self::CarbonMono => "Carbon Mono",
            Self::Bubblegum => "Bubblegum",
            Self::Peacock => "Peacock",
            Self::Marigold => "Marigold",
            Self::Persimmon => "Persimmon",
            Self::PowderBlue => "Powder Blue",
            Self::Pistachio => "Pistachio",
            Self::Buttercream => "Buttercream",
            Self::Sandstone => "Sandstone",
            Self::OatMilk => "Oat Milk",
            Self::PinkLemonade => "Pink Lemonade",
            Self::LagoonMist => "Lagoon Mist",
            Self::SilverLining => "Silver Lining",
            Self::SolarFlare => "Solar Flare",
            Self::BorealSky => "Boreal Sky",
            Self::SapphireRose => "Sapphire Rose",
            Self::CopperTwilight => "Copper Twilight",
            Self::TropicalNight => "Tropical Night",
            Self::AmberTide => "Amber Tide",
            Self::ForestEmber => "Forest Ember",
            Self::PinkVoltage => "Pink Voltage",
            Self::BlueHour => "Blue Hour",
            Self::CitrusNoir => "Citrus Noir",
            Self::CobaltEmber => "Cobalt Ember",
            Self::OpalNight => "Opal Night",
            Self::DesertMirage => "Desert Mirage",
            Self::MoonlitSteel => "Moonlit Steel",
            Self::PeacockFeather => "Peacock Feather",
            Self::RoyalSpectrum => "Royal Spectrum",
            Self::SorbetSky => "Sorbet Sky",
            Self::SeaglassDawn => "Seaglass Dawn",
            Self::LemonPetal => "Lemon Petal",
            Self::PeachLagoon => "Peach Lagoon",
            Self::PearlPrism => "Pearl Prism",
            Self::SkyMeadow => "Sky Meadow",
            Self::CoralMist => "Coral Mist",
            Self::HoneyLavender => "Honey Lavender",
            Self::PoppyRed => "Poppy Red",
            Self::Mandarin => "Mandarin",
            Self::MangoMilk => "Mango Milk",
            Self::Sunflower => "Sunflower",
            Self::Daffodil => "Daffodil",
            Self::LimeWire => "Lime Wire",
            Self::MatchaLatte => "Matcha Latte",
            Self::RacingGreen => "Racing Green",
            Self::Viridian => "Viridian",
            Self::AquaMarine => "Aqua Marine",
            Self::CyanCircuit => "Cyan Circuit",
            Self::IceMint => "Ice Mint",
            Self::RoyalNavy => "Royal Navy",
            Self::Cornflower => "Cornflower",
            Self::FuchsiaPulse => "Fuchsia Pulse",
            Self::SakuraSilk => "Sakura Silk",
            Self::Terracotta => "Terracotta",
            Self::ToffeeCream => "Toffee Cream",
            Self::TrueBlack => "True Black",
            Self::SilverMist => "Silver Mist",
            Self::PureWhite => "Pure White",
            Self::AshStone => "Ash Stone",
            Self::LavenderInk => "Lavender Ink",
            Self::LilacPetal => "Lilac Petal",
            Self::RubySunrise => "Ruby Sunrise",
            Self::MangoWave => "Mango Wave",
            Self::LemonLime => "Lemon Lime",
            Self::JungleCurrent => "Jungle Current",
            Self::CyanAurora => "Cyan Aurora",
            Self::IndigoSpectrum => "Indigo Spectrum",
            Self::FuchsiaLagoon => "Fuchsia Lagoon",
            Self::BronzeOcean => "Bronze Ocean",
            Self::NoirPrism => "Noir Prism",
            Self::CopperRose => "Copper Rose",
            Self::CitrusClouds => "Citrus Clouds",
            Self::AquaBloom => "Aqua Bloom",
            Self::BluePorcelain => "Blue Porcelain",
            Self::RoseGold => "Rose Gold",
            Self::PistachioPeach => "Pistachio Peach",
            Self::ArcticPearl => "Arctic Pearl",
            Self::RubySilk => "Ruby Silk",
            Self::ScarletEmber => "Scarlet Ember",
            Self::GarnetGlow => "Garnet Glow",
            Self::CherryLacquer => "Cherry Lacquer",
            Self::Rosefire => "Rosefire",
            Self::CrimsonTide => "Crimson Tide",
            Self::VelvetRuby => "Velvet Ruby",
            Self::BloodOrange => "Blood Orange",
            Self::StrawberryCream => "Strawberry Cream",
            Self::CherrySorbet => "Cherry Sorbet",
            Self::RoseSatin => "Rose Satin",
            Self::CoralVelvet => "Coral Velvet",
        }
    }
    /// Colour collections include both dark and light palettes.
    pub fn family(self) -> &'static str {
        match self {
            Self::Amethyst => "Purple",
            Self::PurpleRain => "Purple",
            Self::Ultraviolet => "Purple",
            Self::VioletHour => "Purple",
            Self::Nebula => "Purple",
            Self::GrapeSoda => "Purple",
            Self::CyberOrchid => "Purple",
            Self::MidnightLilac => "Purple",
            Self::PlumVelvet => "Pink",
            Self::Blacklight => "Purple",
            Self::Synthwave => "Pink",
            Self::CosmicDusk => "Pink",
            Self::Crimson => "Red",
            Self::CherryBomb => "Red",
            Self::Ruby => "Red",
            Self::Scarlet => "Red",
            Self::BloodMoon => "Red",
            Self::Ember => "Orange",
            Self::RoseNoir => "Pink",
            Self::Inferno => "Orange",
            Self::Burgundy => "Red",
            Self::HotLava => "Orange",
            Self::Arctic => "Cyan",
            Self::DeepOcean => "Blue",
            Self::ElectricBlue => "Blue",
            Self::NorthernLights => "Teal",
            Self::Emerald => "Green",
            Self::ToxicMint => "Green",
            Self::LiquidGold => "Yellow",
            Self::Graphite => "Monochrome",
            Self::LavenderMilk => "Purple",
            Self::LilacCloud => "Purple",
            Self::OrchidGlass => "Purple",
            Self::RoseQuartz => "Pink",
            Self::CherryBlossom => "Red",
            Self::PeachFuzz => "Orange",
            Self::Glacier => "Blue",
            Self::Seafoam => "Teal",
            Self::LemonSorbet => "Yellow",
            Self::Paper => "Monochrome",
            Self::VelvetViolet => "Purple",
            Self::OrchidNight => "Purple",
            Self::ElectricPlum => "Purple",
            Self::Iris => "Purple",
            Self::Dragonfruit => "Pink",
            Self::ObsidianViolet => "Purple",
            Self::Vermilion => "Red",
            Self::RedVelvet => "Red",
            Self::Carmine => "Red",
            Self::CoralFlare => "Orange",
            Self::Merlot => "Red",
            Self::Pomegranate => "Pink",
            Self::JadeTemple => "Green",
            Self::Malachite => "Green",
            Self::TidalTeal => "Teal",
            Self::Sapphire => "Blue",
            Self::Tangerine => "Orange",
            Self::AmberRoom => "Yellow",
            Self::Citron => "Yellow",
            Self::GlacialBlue => "Cyan",
            Self::Wisteria => "Purple",
            Self::BlushSilk => "Pink",
            Self::MintTea => "Green",
            Self::Apricot => "Orange",
            Self::PrismNight => "Gradients",
            Self::VelvetSunset => "Gradients",
            Self::AuroraBorealis => "Gradients",
            Self::MidnightLagoon => "Gradients",
            Self::CrimsonNebula => "Gradients",
            Self::NeonHorizon => "Gradients",
            Self::PurpleHaze => "Gradients",
            Self::CherryAurora => "Gradients",
            Self::DeepSpace => "Gradients",
            Self::MoltenGold => "Gradients",
            Self::EmeraldDream => "Gradients",
            Self::OceanDrive => "Gradients",
            Self::CandyCloud => "Gradients",
            Self::DawnGlow => "Gradients",
            Self::Iridescent => "Gradients",
            Self::Rosewater => "Gradients",
            Self::CobaltInk => "Blue",
            Self::PetrolBlue => "Cyan",
            Self::PacificNight => "Cyan",
            Self::ForestCanopy => "Green",
            Self::PineNeedle => "Green",
            Self::OliveGrove => "Green",
            Self::BronzeAge => "Brown",
            Self::CopperWire => "Brown",
            Self::DesertNight => "Brown",
            Self::Espresso => "Brown",
            Self::SlateSteel => "Monochrome",
            Self::CarbonMono => "Monochrome",
            Self::Bubblegum => "Pink",
            Self::Peacock => "Teal",
            Self::Marigold => "Yellow",
            Self::Persimmon => "Orange",
            Self::PowderBlue => "Blue",
            Self::Pistachio => "Green",
            Self::Buttercream => "Yellow",
            Self::Sandstone => "Brown",
            Self::OatMilk => "Brown",
            Self::PinkLemonade => "Pink",
            Self::LagoonMist => "Cyan",
            Self::SilverLining => "Monochrome",
            Self::SolarFlare => "Gradients",
            Self::BorealSky => "Gradients",
            Self::SapphireRose => "Gradients",
            Self::CopperTwilight => "Gradients",
            Self::TropicalNight => "Gradients",
            Self::AmberTide => "Gradients",
            Self::ForestEmber => "Gradients",
            Self::PinkVoltage => "Gradients",
            Self::BlueHour => "Gradients",
            Self::CitrusNoir => "Gradients",
            Self::CobaltEmber => "Gradients",
            Self::OpalNight => "Gradients",
            Self::DesertMirage => "Gradients",
            Self::MoonlitSteel => "Gradients",
            Self::PeacockFeather => "Gradients",
            Self::RoyalSpectrum => "Gradients",
            Self::SorbetSky => "Gradients",
            Self::SeaglassDawn => "Gradients",
            Self::LemonPetal => "Gradients",
            Self::PeachLagoon => "Gradients",
            Self::PearlPrism => "Gradients",
            Self::SkyMeadow => "Gradients",
            Self::CoralMist => "Gradients",
            Self::HoneyLavender => "Gradients",
            Self::PoppyRed => "Red",
            Self::Mandarin => "Orange",
            Self::MangoMilk => "Orange",
            Self::Sunflower => "Yellow",
            Self::Daffodil => "Yellow",
            Self::LimeWire => "Green",
            Self::MatchaLatte => "Green",
            Self::RacingGreen => "Green",
            Self::Viridian => "Teal",
            Self::AquaMarine => "Teal",
            Self::CyanCircuit => "Cyan",
            Self::IceMint => "Cyan",
            Self::RoyalNavy => "Blue",
            Self::Cornflower => "Blue",
            Self::FuchsiaPulse => "Pink",
            Self::SakuraSilk => "Pink",
            Self::Terracotta => "Brown",
            Self::ToffeeCream => "Brown",
            Self::TrueBlack => "Monochrome",
            Self::SilverMist => "Monochrome",
            Self::PureWhite => "Monochrome",
            Self::AshStone => "Monochrome",
            Self::LavenderInk => "Purple",
            Self::LilacPetal => "Purple",
            Self::RubySunrise => "Gradients",
            Self::MangoWave => "Gradients",
            Self::LemonLime => "Gradients",
            Self::JungleCurrent => "Gradients",
            Self::CyanAurora => "Gradients",
            Self::IndigoSpectrum => "Gradients",
            Self::FuchsiaLagoon => "Gradients",
            Self::BronzeOcean => "Gradients",
            Self::NoirPrism => "Gradients",
            Self::CopperRose => "Gradients",
            Self::CitrusClouds => "Gradients",
            Self::AquaBloom => "Gradients",
            Self::BluePorcelain => "Gradients",
            Self::RoseGold => "Gradients",
            Self::PistachioPeach => "Gradients",
            Self::ArcticPearl => "Gradients",
            Self::RubySilk => "Gradients",
            Self::ScarletEmber => "Gradients",
            Self::GarnetGlow => "Gradients",
            Self::CherryLacquer => "Gradients",
            Self::Rosefire => "Gradients",
            Self::CrimsonTide => "Gradients",
            Self::VelvetRuby => "Gradients",
            Self::BloodOrange => "Gradients",
            Self::StrawberryCream => "Gradients",
            Self::CherrySorbet => "Gradients",
            Self::RoseSatin => "Gradients",
            Self::CoralVelvet => "Gradients",
        }
    }

    /// Colour names and common aliases used by the built-in theme search.
    pub fn colour_keywords(self) -> &'static str {
        match self {
            Self::Amethyst => "purple violet lilac lavender plum indigo dark night after hours",
            Self::PurpleRain => "purple violet lilac lavender plum indigo dark night after hours",
            Self::Ultraviolet => "purple violet lilac lavender plum indigo dark night after hours",
            Self::VioletHour => "purple violet lilac lavender plum indigo dark night after hours",
            Self::Nebula => "purple violet lilac lavender plum indigo dark night after hours",
            Self::GrapeSoda => "purple violet lilac lavender plum indigo dark night after hours",
            Self::CyberOrchid => "purple violet lilac lavender plum indigo dark night after hours",
            Self::MidnightLilac => {
                "purple violet lilac lavender plum indigo dark night after hours"
            }
            Self::PlumVelvet => "pink rose magenta fuchsia dark night after hours",
            Self::Blacklight => "purple violet lilac lavender plum indigo dark night after hours",
            Self::Synthwave => "pink rose magenta fuchsia dark night after hours",
            Self::CosmicDusk => "pink rose magenta fuchsia dark night after hours",
            Self::Crimson => "red crimson ruby scarlet dark night after hours",
            Self::CherryBomb => "red crimson ruby scarlet dark night after hours",
            Self::Ruby => "red crimson ruby scarlet dark night after hours",
            Self::Scarlet => "red crimson ruby scarlet dark night after hours",
            Self::BloodMoon => "red crimson ruby scarlet dark night after hours",
            Self::Ember => "orange tangerine peach apricot dark night after hours",
            Self::RoseNoir => "pink rose magenta fuchsia dark night after hours",
            Self::Inferno => "orange tangerine peach apricot dark night after hours",
            Self::Burgundy => "red crimson ruby scarlet dark night after hours",
            Self::HotLava => "orange tangerine peach apricot dark night after hours",
            Self::Arctic => "cyan aqua aquamarine ice dark night after hours",
            Self::DeepOcean => "blue navy cobalt sapphire ocean dark night after hours",
            Self::ElectricBlue => "blue navy cobalt sapphire ocean dark night after hours",
            Self::NorthernLights => "teal turquoise aqua dark night after hours",
            Self::Emerald => "green emerald forest lime mint dark night after hours",
            Self::ToxicMint => "green emerald forest lime mint chartreuse dark night after hours",
            Self::LiquidGold => "yellow gold golden lemon dark night after hours",
            Self::Graphite => {
                "monochrome neutral black grey gray graphite charcoal dark night after hours"
            }
            Self::LavenderMilk => "purple violet lilac lavender plum indigo light daylight pastel",
            Self::LilacCloud => "purple violet lilac lavender plum indigo light daylight pastel",
            Self::OrchidGlass => {
                "purple violet lilac lavender plum indigo pink fuchsia light daylight pastel"
            }
            Self::RoseQuartz => "pink rose magenta fuchsia light daylight pastel",
            Self::CherryBlossom => "red crimson ruby scarlet light daylight pastel",
            Self::PeachFuzz => "orange tangerine peach apricot light daylight pastel",
            Self::Glacier => "blue navy cobalt sapphire ocean light daylight pastel",
            Self::Seafoam => "teal turquoise aqua light daylight pastel",
            Self::LemonSorbet => "yellow gold golden lemon light daylight pastel",
            Self::Paper => "monochrome neutral white cream ivory paper light daylight pastel",
            Self::VelvetViolet => "purple violet lilac lavender plum indigo dark night after hours",
            Self::OrchidNight => "purple violet lilac lavender plum indigo dark night after hours",
            Self::ElectricPlum => "purple violet lilac lavender plum indigo dark night after hours",
            Self::Iris => "purple violet lilac lavender plum indigo dark night after hours",
            Self::Dragonfruit => "pink rose magenta fuchsia dark night after hours",
            Self::ObsidianViolet => {
                "purple violet lilac lavender plum indigo dark night after hours"
            }
            Self::Vermilion => "red crimson ruby scarlet dark night after hours",
            Self::RedVelvet => "red crimson ruby scarlet dark night after hours",
            Self::Carmine => "red crimson ruby scarlet dark night after hours",
            Self::CoralFlare => "orange tangerine peach apricot dark night after hours",
            Self::Merlot => "red crimson ruby scarlet dark night after hours",
            Self::Pomegranate => "pink rose magenta fuchsia dark night after hours",
            Self::JadeTemple => "green emerald forest lime mint dark night after hours",
            Self::Malachite => "green emerald forest lime mint dark night after hours",
            Self::TidalTeal => "teal turquoise aqua dark night after hours",
            Self::Sapphire => "blue navy cobalt sapphire ocean dark night after hours",
            Self::Tangerine => "orange tangerine peach apricot dark night after hours",
            Self::AmberRoom => "yellow gold golden lemon dark night after hours",
            Self::Citron => "yellow gold golden lemon lime chartreuse dark night after hours",
            Self::GlacialBlue => "cyan aqua aquamarine ice dark night after hours",
            Self::Wisteria => "purple violet lilac lavender plum indigo light daylight pastel",
            Self::BlushSilk => "pink rose magenta fuchsia light daylight pastel",
            Self::MintTea => "green emerald forest lime mint light daylight pastel",
            Self::Apricot => "orange tangerine peach apricot light daylight pastel",
            Self::PrismNight => {
                "gradient gradients multicolour multicolor blue navy cobalt sapphire ocean pink rose magenta fuchsia cyan aqua aquamarine ice purple violet lilac lavender plum indigo dark night after hours"
            }
            Self::VelvetSunset => {
                "gradient gradients multicolour multicolor pink rose magenta fuchsia orange tangerine peach apricot dark night after hours"
            }
            Self::AuroraBorealis => {
                "gradient gradients multicolour multicolor cyan aqua aquamarine ice teal turquoise purple violet lilac lavender plum indigo green emerald forest lime mint dark night after hours"
            }
            Self::MidnightLagoon => {
                "gradient gradients multicolour multicolor cyan aqua aquamarine ice blue navy cobalt sapphire ocean dark night after hours"
            }
            Self::CrimsonNebula => {
                "gradient gradients multicolour multicolor pink rose magenta fuchsia blue navy cobalt sapphire ocean dark night after hours"
            }
            Self::NeonHorizon => {
                "gradient gradients multicolour multicolor blue navy cobalt sapphire ocean purple violet lilac lavender plum indigo cyan aqua aquamarine ice pink rose magenta fuchsia dark night after hours"
            }
            Self::PurpleHaze => {
                "gradient gradients multicolour multicolor purple violet lilac lavender plum indigo blue navy cobalt sapphire ocean dark night after hours"
            }
            Self::CherryAurora => {
                "gradient gradients multicolour multicolor pink rose magenta fuchsia blue navy cobalt sapphire ocean dark night after hours"
            }
            Self::DeepSpace => {
                "gradient gradients multicolour multicolor blue navy cobalt sapphire ocean cyan aqua aquamarine ice dark night after hours"
            }
            Self::MoltenGold => {
                "gradient gradients multicolour multicolor orange tangerine peach apricot pink rose magenta fuchsia dark night after hours"
            }
            Self::EmeraldDream => {
                "gradient gradients multicolour multicolor green emerald forest lime mint teal turquoise aqua blue navy cobalt sapphire ocean dark night after hours"
            }
            Self::OceanDrive => {
                "gradient gradients multicolour multicolor blue navy cobalt sapphire ocean cyan aqua aquamarine ice purple violet lilac lavender plum indigo dark night after hours"
            }
            Self::CandyCloud => {
                "gradient gradients multicolour multicolor purple violet lilac lavender plum indigo pink rose magenta fuchsia blue navy cobalt sapphire ocean light daylight pastel"
            }
            Self::DawnGlow => {
                "gradient gradients multicolour multicolor orange tangerine peach apricot red crimson ruby scarlet purple violet lilac lavender plum indigo light daylight pastel"
            }
            Self::Iridescent => {
                "gradient gradients multicolour multicolor cyan aqua aquamarine ice purple violet lilac lavender plum indigo green emerald forest lime mint blue navy cobalt sapphire ocean light daylight pastel"
            }
            Self::Rosewater => {
                "gradient gradients multicolour multicolor pink rose magenta fuchsia purple violet lilac lavender plum indigo orange tangerine peach apricot light daylight pastel"
            }
            Self::CobaltInk => "blue navy cobalt sapphire ocean dark night after hours",
            Self::PetrolBlue => "cyan aqua aquamarine ice dark night after hours",
            Self::PacificNight => "cyan aqua aquamarine ice dark night after hours",
            Self::ForestCanopy => "green emerald forest lime mint dark night after hours",
            Self::PineNeedle => "green emerald forest lime mint dark night after hours",
            Self::OliveGrove => {
                "green emerald forest lime mint olive yellow dark night after hours"
            }
            Self::BronzeAge => "brown beige tan copper bronze coffee cream dark night after hours",
            Self::CopperWire => "brown beige tan copper bronze coffee cream dark night after hours",
            Self::DesertNight => {
                "brown beige tan copper bronze coffee cream dark night after hours"
            }
            Self::Espresso => "brown beige tan copper bronze coffee cream dark night after hours",
            Self::SlateSteel => "monochrome neutral grey gray blue steel dark night after hours",
            Self::CarbonMono => {
                "monochrome neutral black grey gray charcoal dark night after hours"
            }
            Self::Bubblegum => "pink rose magenta fuchsia dark night after hours",
            Self::Peacock => "teal turquoise aqua green dark night after hours",
            Self::Marigold => "yellow gold golden lemon dark night after hours",
            Self::Persimmon => "orange tangerine peach apricot dark night after hours",
            Self::PowderBlue => "blue navy cobalt sapphire ocean light daylight pastel",
            Self::Pistachio => "green emerald forest lime mint light daylight pastel",
            Self::Buttercream => "yellow gold golden lemon light daylight pastel",
            Self::Sandstone => "brown beige tan copper bronze coffee cream light daylight pastel",
            Self::OatMilk => "brown beige tan copper bronze coffee cream light daylight pastel",
            Self::PinkLemonade => "pink rose magenta fuchsia light daylight pastel",
            Self::LagoonMist => "cyan aqua aquamarine ice light daylight pastel",
            Self::SilverLining => "monochrome neutral white grey gray silver light daylight pastel",
            Self::SolarFlare => {
                "gradient gradients multicolour multicolor red crimson ruby scarlet orange tangerine peach apricot pink rose magenta fuchsia dark night after hours"
            }
            Self::BorealSky => {
                "gradient gradients multicolour multicolor teal turquoise aqua cyan aquamarine ice blue navy cobalt sapphire ocean green emerald forest lime mint dark night after hours"
            }
            Self::SapphireRose => {
                "gradient gradients multicolour multicolor blue navy cobalt sapphire ocean purple violet lilac lavender plum indigo pink rose magenta fuchsia dark night after hours"
            }
            Self::CopperTwilight => {
                "gradient gradients multicolour multicolor orange tangerine peach apricot pink rose magenta fuchsia cyan aqua aquamarine ice dark night after hours"
            }
            Self::TropicalNight => {
                "gradient gradients multicolour multicolor teal turquoise aqua green emerald forest lime mint purple violet lilac lavender plum indigo dark night after hours"
            }
            Self::AmberTide => {
                "gradient gradients multicolour multicolor yellow gold golden lemon teal turquoise aqua blue navy cobalt sapphire ocean orange tangerine peach apricot dark night after hours"
            }
            Self::ForestEmber => {
                "gradient gradients multicolour multicolor green emerald forest lime mint orange tangerine peach apricot pink rose magenta fuchsia dark night after hours"
            }
            Self::PinkVoltage => {
                "gradient gradients multicolour multicolor pink rose magenta fuchsia purple violet lilac lavender plum indigo cyan aqua aquamarine ice dark night after hours"
            }
            Self::BlueHour => {
                "gradient gradients multicolour multicolor blue navy cobalt sapphire ocean cyan aqua aquamarine ice dark night after hours"
            }
            Self::CitrusNoir => {
                "gradient gradients multicolour multicolor yellow gold golden lemon green emerald forest lime mint blue navy cobalt sapphire ocean dark night after hours"
            }
            Self::CobaltEmber => {
                "gradient gradients multicolour multicolor blue navy cobalt sapphire ocean purple violet lilac lavender plum indigo orange tangerine peach apricot dark night after hours"
            }
            Self::OpalNight => {
                "gradient gradients multicolour multicolor teal turquoise aqua purple violet lilac lavender plum indigo blue navy cobalt sapphire ocean dark night after hours"
            }
            Self::DesertMirage => {
                "gradient gradients multicolour multicolor orange tangerine peach apricot pink rose magenta fuchsia blue navy cobalt sapphire ocean dark night after hours"
            }
            Self::MoonlitSteel => {
                "gradient gradients multicolour multicolor blue navy cobalt sapphire ocean purple violet lilac lavender plum indigo neutral grey gray black silver dark night after hours"
            }
            Self::PeacockFeather => {
                "gradient gradients multicolour multicolor green emerald forest lime mint cyan aqua aquamarine ice blue navy cobalt sapphire ocean dark night after hours"
            }
            Self::RoyalSpectrum => {
                "gradient gradients multicolour multicolor purple violet lilac lavender plum indigo blue navy cobalt sapphire ocean teal turquoise aqua dark night after hours"
            }
            Self::SorbetSky => {
                "gradient gradients multicolour multicolor orange tangerine peach apricot pink rose magenta fuchsia blue navy cobalt sapphire ocean red crimson ruby scarlet light daylight pastel"
            }
            Self::SeaglassDawn => {
                "gradient gradients multicolour multicolor green emerald forest lime mint cyan aqua aquamarine ice purple violet lilac lavender plum indigo teal turquoise light daylight pastel"
            }
            Self::LemonPetal => {
                "gradient gradients multicolour multicolor yellow gold golden lemon pink rose magenta fuchsia green emerald forest lime mint light daylight pastel"
            }
            Self::PeachLagoon => {
                "gradient gradients multicolour multicolor orange tangerine peach apricot green emerald forest lime mint yellow gold golden lemon light daylight pastel"
            }
            Self::PearlPrism => {
                "gradient gradients multicolour multicolor orange tangerine peach apricot blue navy cobalt sapphire ocean pink rose magenta fuchsia neutral grey gray silver white light daylight pastel"
            }
            Self::SkyMeadow => {
                "gradient gradients multicolour multicolor blue navy cobalt sapphire ocean green emerald forest lime mint orange tangerine peach apricot light daylight pastel"
            }
            Self::CoralMist => {
                "gradient gradients multicolour multicolor red crimson ruby scarlet pink rose magenta fuchsia purple violet lilac lavender plum indigo light daylight pastel"
            }
            Self::HoneyLavender => {
                "gradient gradients multicolour multicolor orange tangerine peach apricot purple violet lilac lavender plum indigo green emerald forest lime mint light daylight pastel"
            }
            Self::PoppyRed => "red crimson ruby scarlet dark night after hours",
            Self::Mandarin => "orange tangerine peach apricot dark night after hours",
            Self::MangoMilk => "orange tangerine peach apricot light daylight pastel",
            Self::Sunflower => "yellow gold golden lemon dark night after hours",
            Self::Daffodil => "yellow gold golden lemon light daylight pastel",
            Self::LimeWire => "green emerald forest lime mint dark night after hours",
            Self::MatchaLatte => "green emerald forest lime mint light daylight pastel",
            Self::RacingGreen => "green emerald forest lime mint dark night after hours",
            Self::Viridian => "teal turquoise aqua dark night after hours",
            Self::AquaMarine => "teal turquoise aqua light daylight pastel",
            Self::CyanCircuit => "cyan aqua aquamarine ice dark night after hours",
            Self::IceMint => "cyan aqua aquamarine ice light daylight pastel",
            Self::RoyalNavy => "blue navy cobalt sapphire ocean dark night after hours",
            Self::Cornflower => "blue navy cobalt sapphire ocean light daylight pastel",
            Self::FuchsiaPulse => "pink rose magenta fuchsia dark night after hours",
            Self::SakuraSilk => "pink rose magenta fuchsia light daylight pastel",
            Self::Terracotta => "brown beige tan copper bronze coffee cream dark night after hours",
            Self::ToffeeCream => "brown beige tan copper bronze coffee cream light daylight pastel",
            Self::TrueBlack => "monochrome neutral black charcoal dark night after hours",
            Self::SilverMist => "monochrome neutral white grey gray silver light daylight pastel",
            Self::PureWhite => "monochrome neutral white light daylight pastel",
            Self::AshStone => {
                "monochrome neutral black grey gray ash charcoal dark night after hours"
            }
            Self::LavenderInk => "purple violet lilac lavender plum indigo dark night after hours",
            Self::LilacPetal => "purple violet lilac lavender plum indigo light daylight pastel",
            Self::RubySunrise => {
                "gradient gradients multicolour multicolor pink rose magenta fuchsia orange tangerine peach apricot yellow gold golden lemon red crimson ruby scarlet dark night after hours"
            }
            Self::MangoWave => {
                "gradient gradients multicolour multicolor orange tangerine peach apricot teal turquoise aqua dark night after hours"
            }
            Self::LemonLime => {
                "gradient gradients multicolour multicolor yellow gold golden lemon green emerald forest lime mint cyan aqua aquamarine ice dark night after hours"
            }
            Self::JungleCurrent => {
                "gradient gradients multicolour multicolor green emerald forest lime mint teal turquoise aqua blue navy cobalt sapphire ocean dark night after hours"
            }
            Self::CyanAurora => {
                "gradient gradients multicolour multicolor cyan aqua aquamarine ice blue navy cobalt sapphire ocean pink rose magenta fuchsia dark night after hours"
            }
            Self::IndigoSpectrum => {
                "gradient gradients multicolour multicolor blue navy cobalt sapphire ocean purple violet lilac lavender plum indigo pink rose magenta fuchsia dark night after hours"
            }
            Self::FuchsiaLagoon => {
                "gradient gradients multicolour multicolor pink rose magenta fuchsia purple violet lilac lavender plum indigo teal turquoise aqua dark night after hours"
            }
            Self::BronzeOcean => {
                "gradient gradients multicolour multicolor orange tangerine peach apricot cyan aqua aquamarine ice dark night after hours"
            }
            Self::NoirPrism => {
                "gradient gradients multicolour multicolor blue navy cobalt sapphire ocean purple violet lilac lavender plum indigo neutral grey gray black silver dark night after hours"
            }
            Self::CopperRose => {
                "gradient gradients multicolour multicolor orange tangerine peach apricot pink rose magenta fuchsia dark night after hours"
            }
            Self::CitrusClouds => {
                "gradient gradients multicolour multicolor yellow gold golden lemon orange tangerine peach apricot green emerald forest lime mint light daylight pastel"
            }
            Self::AquaBloom => {
                "gradient gradients multicolour multicolor cyan aqua aquamarine ice green emerald forest lime mint pink rose magenta fuchsia light daylight pastel"
            }
            Self::BluePorcelain => {
                "gradient gradients multicolour multicolor blue navy cobalt sapphire ocean monochrome neutral grey gray purple violet lilac lavender plum indigo light daylight pastel"
            }
            Self::RoseGold => {
                "gradient gradients multicolour multicolor pink rose magenta fuchsia orange tangerine peach apricot light daylight pastel"
            }
            Self::PistachioPeach => {
                "gradient gradients multicolour multicolor green emerald forest lime mint yellow gold golden lemon orange tangerine peach apricot light daylight pastel"
            }
            Self::ArcticPearl => {
                "gradient gradients multicolour multicolor monochrome neutral grey gray cyan aqua aquamarine ice blue navy cobalt sapphire ocean silver white light daylight pastel"
            }
            Self::RubySilk => {
                "gradient gradients multicolour multicolor red ruby wine burgundy rose dark night after hours"
            }
            Self::ScarletEmber => {
                "gradient gradients multicolour multicolor red scarlet ember orange vermilion dark night after hours"
            }
            Self::GarnetGlow => {
                "gradient gradients multicolour multicolor red garnet wine burgundy cherry dark night after hours"
            }
            Self::CherryLacquer => {
                "gradient gradients multicolour multicolor red cherry lacquer ruby scarlet dark night after hours"
            }
            Self::Rosefire => {
                "gradient gradients multicolour multicolor red rose fire ember orange dark night after hours"
            }
            Self::CrimsonTide => {
                "gradient gradients multicolour multicolor red crimson cherry carmine wine dark night after hours"
            }
            Self::VelvetRuby => {
                "gradient gradients multicolour multicolor red ruby velvet garnet burgundy dark night after hours"
            }
            Self::BloodOrange => {
                "gradient gradients multicolour multicolor red blood orange vermilion ember dark night after hours"
            }
            Self::StrawberryCream => {
                "gradient gradients multicolour multicolor red strawberry cherry cream pink rose light daylight pastel"
            }
            Self::CherrySorbet => {
                "gradient gradients multicolour multicolor red cherry sorbet strawberry pink rose light daylight pastel"
            }
            Self::RoseSatin => {
                "gradient gradients multicolour multicolor red rose satin ruby pink coral light daylight pastel"
            }
            Self::CoralVelvet => {
                "gradient gradients multicolour multicolor red coral velvet scarlet orange peach light daylight pastel"
            }
        }
    }
    /// Three opaque canvas colours, drawn as a real gradient by the native
    /// shell and previews. Controls keep the matching solid palette so their
    /// contrast does not depend on where they fall across the canvas.
    /// Saved identities resolve these stops without a file or a cached image.
    pub fn gradient(self) -> Option<[Color32; 3]> {
        let stops = match self {
            Self::PrismNight => [0x110b24, 0x29102c, 0x082a30],
            Self::VelvetSunset => [0x24101f, 0x35152c, 0x2b2110],
            Self::AuroraBorealis => [0x081c20, 0x132a28, 0x211439],
            Self::MidnightLagoon => [0x071a25, 0x082a30, 0x181639],
            Self::CrimsonNebula => [0x220d21, 0x2e1020, 0x1a1538],
            Self::NeonHorizon => [0x12142a, 0x271531, 0x0c2b34],
            Self::PurpleHaze => [0x1a102c, 0x29123a, 0x14243c],
            Self::CherryAurora => [0x250e19, 0x321529, 0x141d36],
            Self::DeepSpace => [0x0a1024, 0x17143a, 0x0a2a34],
            Self::MoltenGold => [0x21170a, 0x312011, 0x291322],
            Self::EmeraldDream => [0x0b2019, 0x112f2c, 0x19233c],
            Self::OceanDrive => [0x0a172b, 0x132e39, 0x2a1740],
            Self::CandyCloud => [0xf7eaff, 0xffecf3, 0xe6f4ff],
            Self::DawnGlow => [0xfff1df, 0xffe7e8, 0xf7edff],
            Self::Iridescent => [0xe9f7ff, 0xf1ebff, 0xe4f9ef],
            Self::Rosewater => [0xfff0f5, 0xf5e9fa, 0xffefdf],
            Self::SolarFlare => [0x23120d, 0x3a2812, 0x301122],
            Self::BorealSky => [0x0a201c, 0x10343a, 0x172246],
            Self::SapphireRose => [0x0d1632, 0x271644, 0x361a30],
            Self::CopperTwilight => [0x261a13, 0x321b32, 0x122b3c],
            Self::TropicalNight => [0x082420, 0x153525, 0x311940],
            Self::AmberTide => [0x24200c, 0x0d3232, 0x152542],
            Self::ForestEmber => [0x122311, 0x302815, 0x361823],
            Self::PinkVoltage => [0x2b102b, 0x27153e, 0x0d2c3d],
            Self::BlueHour => [0x0d1b35, 0x10343e, 0x25243a],
            Self::CitrusNoir => [0x20230c, 0x12352c, 0x17263e],
            Self::CobaltEmber => [0x0d1734, 0x30203d, 0x3b2418],
            Self::OpalNight => [0x122523, 0x2c1d3e, 0x152d45],
            Self::DesertMirage => [0x261e12, 0x351e2b, 0x172c40],
            Self::MoonlitSteel => [0x141c29, 0x25313b, 0x251d35],
            Self::PeacockFeather => [0x0a231c, 0x12333f, 0x261e44],
            Self::RoyalSpectrum => [0x1d1039, 0x15233f, 0x10372f],
            Self::SorbetSky => [0xffefd9, 0xffe6ef, 0xe4f2ff],
            Self::SeaglassDawn => [0xe8f8ef, 0xe4f3fa, 0xf0e8ff],
            Self::LemonPetal => [0xfbf8dc, 0xffeaf0, 0xe9f5e5],
            Self::PeachLagoon => [0xffeddf, 0xe1f5ef, 0xfaf2da],
            Self::PearlPrism => [0xf7f4ec, 0xe5f0ff, 0xfbe8f5],
            Self::SkyMeadow => [0xe8f3ff, 0xe7f6e7, 0xfff2dc],
            Self::CoralMist => [0xffede8, 0xf8e7f0, 0xeee8ff],
            Self::HoneyLavender => [0xfff4dc, 0xf4e7ff, 0xe6f4ee],
            Self::RubySunrise => [0x240c16, 0x382015, 0x2a2811],
            Self::MangoWave => [0x25190c, 0x352b10, 0x0c3031],
            Self::LemonLime => [0x24250a, 0x14341d, 0x0a2c36],
            Self::JungleCurrent => [0x0b2413, 0x0d332d, 0x122b46],
            Self::CyanAurora => [0x07242d, 0x172c48, 0x321a36],
            Self::IndigoSpectrum => [0x111436, 0x291342, 0x341424],
            Self::FuchsiaLagoon => [0x2b1028, 0x291c40, 0x0e3130],
            Self::BronzeOcean => [0x251c13, 0x312917, 0x122b3b],
            Self::NoirPrism => [0x090a0c, 0x202733, 0x251b33],
            Self::CopperRose => [0x261810, 0x32221b, 0x321a2c],
            Self::CitrusClouds => [0xfff9dc, 0xffead6, 0xe8f6df],
            Self::AquaBloom => [0xe4f8ff, 0xe4f8ed, 0xfbe5f1],
            Self::BluePorcelain => [0xe8f1ff, 0xf7f5f1, 0xf0e6ff],
            Self::RoseGold => [0xffe9f1, 0xffeddf, 0xfff6dc],
            Self::PistachioPeach => [0xecf7df, 0xfff7dd, 0xffe8dd],
            Self::ArcticPearl => [0xf9fcff, 0xe5f5fa, 0xeaeef5],
            Self::RubySilk => [0x250b15, 0x4c142a, 0x311015],
            Self::ScarletEmber => [0x230c09, 0x531c10, 0x351017],
            Self::GarnetGlow => [0x200a0d, 0x42101a, 0x541e2c],
            Self::CherryLacquer => [0x27080e, 0x571522, 0x330c15],
            Self::Rosefire => [0x270c13, 0x4c1a26, 0x421c0e],
            Self::CrimsonTide => [0x220a0b, 0x4b1b25, 0x34101e],
            Self::VelvetRuby => [0x1f0d10, 0x481d2c, 0x35110d],
            Self::BloodOrange => [0x280c07, 0x56250f, 0x401014],
            Self::StrawberryCream => [0xffeceb, 0xffdce0, 0xfff2e8],
            Self::CherrySorbet => [0xffe8eb, 0xfbd8dd, 0xffede2],
            Self::RoseSatin => [0xffeff0, 0xf6d9db, 0xffe6df],
            Self::CoralVelvet => [0xffeddf, 0xffddd6, 0xffedf0],
            _ => return None,
        };
        Some(stops.map(|hex| Color32::from_rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)))
    }

    pub fn palette(self) -> Palette {
        let (dark, window, panel, surface, accent) = match self {
            Self::Amethyst => (true, 0x100b1d, 0x171027, 0x211833, 0xc29aff),
            Self::PurpleRain => (true, 0x150d20, 0x1e142c, 0x2b1d3d, 0xd4a1ff),
            Self::Ultraviolet => (true, 0x0d081b, 0x160e2b, 0x231642, 0xb295ff),
            Self::VioletHour => (true, 0x181221, 0x231a30, 0x30243e, 0xdfadff),
            Self::Nebula => (true, 0x0d1023, 0x141a32, 0x202746, 0xc7a1ff),
            Self::GrapeSoda => (true, 0x1b0c24, 0x281332, 0x391d47, 0xe7a2ff),
            Self::CyberOrchid => (true, 0x100f22, 0x19182f, 0x272442, 0xef8cff),
            Self::MidnightLilac => (true, 0x11101a, 0x1a1825, 0x282537, 0xbdb2ff),
            Self::PlumVelvet => (true, 0x1e101a, 0x2a1826, 0x3b2335, 0xedafd6),
            Self::Blacklight => (true, 0x08080e, 0x11101b, 0x1d192b, 0xaa9aff),
            Self::Synthwave => (true, 0x160d23, 0x211334, 0x311e49, 0xff8cdb),
            Self::CosmicDusk => (true, 0x16101e, 0x22182a, 0x33243c, 0xffb4e0),
            Self::Crimson => (true, 0x1b0b10, 0x271117, 0x381c25, 0xff879e),
            Self::CherryBomb => (true, 0x210c16, 0x301322, 0x441e30, 0xff86b5),
            Self::Ruby => (true, 0x190d17, 0x271320, 0x391d30, 0xff83ba),
            Self::Scarlet => (true, 0x1c0e0e, 0x2a1717, 0x3d2222, 0xff8d85),
            Self::BloodMoon => (true, 0x10090b, 0x1c1014, 0x2b1a20, 0xf988a1),
            Self::Ember => (true, 0x1c100b, 0x291a12, 0x3b271d, 0xffac7d),
            Self::RoseNoir => (true, 0x180f13, 0x24181e, 0x34242d, 0xf7a2b8),
            Self::Inferno => (true, 0x1b0d0a, 0x291610, 0x3b2219, 0xff996e),
            Self::Burgundy => (true, 0x1c0d1b, 0x2b1729, 0x3e223a, 0xe99bcb),
            Self::HotLava => (true, 0x17100f, 0x241a18, 0x352724, 0xffa18b),
            Self::Arctic => (true, 0x09151d, 0x10212b, 0x1b303c, 0x7edbff),
            Self::DeepOcean => (true, 0x08131c, 0x0f202d, 0x19303f, 0x7acfff),
            Self::ElectricBlue => (true, 0x0c1122, 0x141d33, 0x212d48, 0x93b7ff),
            Self::NorthernLights => (true, 0x0b181b, 0x11262b, 0x1c383e, 0x81e8d9),
            Self::Emerald => (true, 0x0b1814, 0x13261f, 0x20382e, 0x8ce4b2),
            Self::ToxicMint => (true, 0x10170c, 0x1b2514, 0x29371f, 0xb4ee8a),
            Self::LiquidGold => (true, 0x19150c, 0x272116, 0x393123, 0xf3ce83),
            Self::Graphite => (true, 0x101114, 0x191b20, 0x272a31, 0xc6cbd8),
            Self::LavenderMilk => (false, 0xf4efff, 0xfcf9ff, 0xe9e0f7, 0x7141ae),
            Self::LilacCloud => (false, 0xf6f2fc, 0xffffff, 0xeae3f5, 0x76529e),
            Self::OrchidGlass => (false, 0xfcf0fa, 0xfffaff, 0xf2dff0, 0x9b3988),
            Self::RoseQuartz => (false, 0xfff1f4, 0xfffafb, 0xf8e0e8, 0xb32e5a),
            Self::CherryBlossom => (false, 0xfff2ef, 0xfffcfb, 0xf9e2dc, 0xb53643),
            Self::PeachFuzz => (false, 0xfff3e9, 0xfffdf9, 0xf5e4d5, 0xa14a23),
            Self::Glacier => (false, 0xedf7ff, 0xfbfdff, 0xdcecf6, 0x20618e),
            Self::Seafoam => (false, 0xedf9f5, 0xfbfffd, 0xdceee6, 0x246c59),
            Self::LemonSorbet => (false, 0xfaf8e9, 0xfffef7, 0xefecd4, 0x746018),
            Self::Paper => (false, 0xf7f5f2, 0xffffff, 0xeae6e1, 0x66517b),
            Self::VelvetViolet => (true, 0x1b112a, 0x251a33, 0x352743, 0xd9afff),
            Self::OrchidNight => (true, 0x170e24, 0x23152f, 0x342141, 0xf2a4f5),
            Self::ElectricPlum => (true, 0x190b26, 0x251236, 0x371d49, 0xdd9dff),
            Self::Iris => (true, 0x10132a, 0x181d37, 0x262c4a, 0xb5baff),
            Self::Dragonfruit => (true, 0x210d25, 0x2e1530, 0x40213f, 0xff9ae4),
            Self::ObsidianViolet => (true, 0x0c0a13, 0x15111f, 0x231b32, 0xc9a6ff),
            Self::Vermilion => (true, 0x210e0d, 0x2d1714, 0x3e241f, 0xffa68d),
            Self::RedVelvet => (true, 0x220e17, 0x301722, 0x422331, 0xff9bbd),
            Self::Carmine => (true, 0x210c16, 0x2e1420, 0x412030, 0xff8fae),
            Self::CoralFlare => (true, 0x22120e, 0x301c18, 0x422a24, 0xffb19c),
            Self::Merlot => (true, 0x1b0c1b, 0x281429, 0x3b213a, 0xf2a0dd),
            Self::Pomegranate => (true, 0x210d1d, 0x2f1629, 0x42243a, 0xff9bc9),
            Self::JadeTemple => (true, 0x0a1d17, 0x122a22, 0x203c30, 0x80e4b6),
            Self::Malachite => (true, 0x0e1e11, 0x182b1b, 0x273d29, 0xb0f08d),
            Self::TidalTeal => (true, 0x081d22, 0x102a30, 0x1c3b41, 0x80e5e1),
            Self::Sapphire => (true, 0x0a142b, 0x121f3c, 0x202f50, 0xa5c2ff),
            Self::Tangerine => (true, 0x21170d, 0x2e2115, 0x403220, 0xffc181),
            Self::AmberRoom => (true, 0x201b0d, 0x2c2615, 0x3d3522, 0xfbd895),
            Self::Citron => (true, 0x191d0c, 0x232b14, 0x333c22, 0xd2ec7d),
            Self::GlacialBlue => (true, 0x091b27, 0x102936, 0x1d3948, 0x92e0ff),
            Self::Wisteria => (false, 0xf4edff, 0xfcf9ff, 0xe7dcf6, 0x7444aa),
            Self::BlushSilk => (false, 0xfff0f6, 0xfffafd, 0xf5dfeb, 0xa33465),
            Self::MintTea => (false, 0xeff9e8, 0xfcfff9, 0xe0edda, 0x386c2b),
            Self::Apricot => (false, 0xfff2e3, 0xfffaf3, 0xf5e1ca, 0x9b4c1e),
            Self::PrismNight => (true, 0x110b24, 0x171329, 0x24203b, 0xcba6ff),
            Self::VelvetSunset => (true, 0x24101f, 0x281b27, 0x3b2937, 0xffb89d),
            Self::AuroraBorealis => (true, 0x081c20, 0x10282a, 0x1c3938, 0x87f1cb),
            Self::MidnightLagoon => (true, 0x071a25, 0x102532, 0x1c3643, 0x7de1f4),
            Self::CrimsonNebula => (true, 0x220d21, 0x2c1727, 0x3e253a, 0xff99ca),
            Self::NeonHorizon => (true, 0x12142a, 0x1a1e33, 0x292d45, 0xf4a2ff),
            Self::PurpleHaze => (true, 0x1a102c, 0x231931, 0x322741, 0xd7b0ff),
            Self::CherryAurora => (true, 0x250e19, 0x301a25, 0x402936, 0xffa0ba),
            Self::DeepSpace => (true, 0x0a1024, 0x121b2e, 0x202b40, 0xa6b9ff),
            Self::MoltenGold => (true, 0x21170a, 0x2b2113, 0x3c311f, 0xffd283),
            Self::EmeraldDream => (true, 0x0b2019, 0x142a22, 0x233c2e, 0x98f3be),
            Self::OceanDrive => (true, 0x0a172b, 0x11233a, 0x20334a, 0x8bd8ff),
            Self::CandyCloud => (false, 0xf7eaff, 0xfffaff, 0xeadcf5, 0x7441ad),
            Self::DawnGlow => (false, 0xfff1df, 0xfffcf7, 0xf4e1d9, 0xa54142),
            Self::Iridescent => (false, 0xe9f7ff, 0xf9fdff, 0xdceaf6, 0x3e5e9d),
            Self::Rosewater => (false, 0xfff0f5, 0xfffafd, 0xf4dfed, 0xa02f68),
            Self::CobaltInk => (true, 0x0b1328, 0x131e37, 0x202d49, 0x91b8ff),
            Self::PetrolBlue => (true, 0x0a1c23, 0x132a33, 0x203b43, 0x88d9df),
            Self::PacificNight => (true, 0x081a29, 0x102735, 0x1b3747, 0x79d6f5),
            Self::ForestCanopy => (true, 0x101c13, 0x192a1d, 0x293c2c, 0xa4d994),
            Self::PineNeedle => (true, 0x0c1b18, 0x152922, 0x233b32, 0x89cfae),
            Self::OliveGrove => (true, 0x1a1c10, 0x262a18, 0x343a23, 0xd1db8a),
            Self::BronzeAge => (true, 0x1d1710, 0x2a2218, 0x3b3125, 0xdcc08f),
            Self::CopperWire => (true, 0x22150f, 0x302018, 0x412e24, 0xf2b18d),
            Self::DesertNight => (true, 0x201b15, 0x2d271e, 0x3d362a, 0xe1c6a5),
            Self::Espresso => (true, 0x1b1411, 0x281f1a, 0x392e27, 0xd5b5a0),
            Self::SlateSteel => (true, 0x11191f, 0x1c252e, 0x2b3640, 0xb0ccd9),
            Self::CarbonMono => (true, 0x121313, 0x1e2020, 0x2d3030, 0xd7dfdb),
            Self::Bubblegum => (true, 0x221022, 0x301b30, 0x422a3e, 0xffa6d4),
            Self::Peacock => (true, 0x081d1e, 0x112c2c, 0x203d3a, 0x81e2cc),
            Self::Marigold => (true, 0x211c0b, 0x2f2814, 0x403822, 0xf6d477),
            Self::Persimmon => (true, 0x23150f, 0x321f19, 0x432d26, 0xffb198),
            Self::PowderBlue => (false, 0xedf5ff, 0xfafcff, 0xdce7f4, 0x345f9e),
            Self::Pistachio => (false, 0xf2f8e9, 0xfcfef8, 0xe3ecd5, 0x4a6b26),
            Self::Buttercream => (false, 0xfff8df, 0xfffdf3, 0xf0e6c6, 0x7c5b16),
            Self::Sandstone => (false, 0xf8f0e5, 0xfffbf5, 0xeadfce, 0x805932),
            Self::OatMilk => (false, 0xf7f4ec, 0xfffdf8, 0xe9e4d8, 0x6b6040),
            Self::PinkLemonade => (false, 0xffeff4, 0xfffafc, 0xf2dce6, 0xa43862),
            Self::LagoonMist => (false, 0xeaf8f8, 0xf8fefe, 0xd5ebeb, 0x246b70),
            Self::SilverLining => (false, 0xf0f3f6, 0xfcfdff, 0xdfe5eb, 0x4b607d),
            Self::SolarFlare => (true, 0x23120d, 0x2d2018, 0x3e3024, 0xffbf82),
            Self::BorealSky => (true, 0x0a201c, 0x132b2c, 0x223c3d, 0x8af0d1),
            Self::SapphireRose => (true, 0x0d1632, 0x181f36, 0x282f48, 0xb8b9ff),
            Self::CopperTwilight => (true, 0x261a13, 0x30231f, 0x41322c, 0xf1bd9b),
            Self::TropicalNight => (true, 0x082420, 0x132d2b, 0x223d3a, 0x85e6bb),
            Self::AmberTide => (true, 0x24200c, 0x2e2917, 0x3f3825, 0xf3d58b),
            Self::ForestEmber => (true, 0x122311, 0x1d2c1d, 0x2c3c2b, 0xb6e39d),
            Self::PinkVoltage => (true, 0x2b102b, 0x311d31, 0x422d40, 0xffa7e9),
            Self::BlueHour => (true, 0x0d1b35, 0x17253b, 0x26364c, 0xa8cbff),
            Self::CitrusNoir => (true, 0x20230c, 0x292d16, 0x353b22, 0xdbeb8c),
            Self::CobaltEmber => (true, 0x0d1734, 0x182239, 0x28334b, 0xa9c6ff),
            Self::OpalNight => (true, 0x122523, 0x1b2e2e, 0x283b39, 0x9be6d8),
            Self::DesertMirage => (true, 0x261e12, 0x30281e, 0x41372d, 0xeed0a5),
            Self::MoonlitSteel => (true, 0x141c29, 0x202936, 0x2d3844, 0xc6d3e5),
            Self::PeacockFeather => (true, 0x0a231c, 0x142e28, 0x253e37, 0x94e7bd),
            Self::RoyalSpectrum => (true, 0x1d1039, 0x251b37, 0x352b48, 0xd7b6ff),
            Self::SorbetSky => (false, 0xffefd9, 0xfffbf6, 0xf2e2da, 0x9e4c45),
            Self::SeaglassDawn => (false, 0xe8f8ef, 0xf9fefa, 0xdaece7, 0x286b62),
            Self::LemonPetal => (false, 0xfbf8dc, 0xfffef6, 0xece7c9, 0x75601c),
            Self::PeachLagoon => (false, 0xffeddf, 0xfffaf4, 0xf0dfd2, 0x9b4d32),
            Self::PearlPrism => (false, 0xf7f4ec, 0xfffdfb, 0xe8e3dc, 0x735476),
            Self::SkyMeadow => (false, 0xe8f3ff, 0xfafdfc, 0xdbe9ed, 0x34628c),
            Self::CoralMist => (false, 0xffede8, 0xfffaf8, 0xf2ddd9, 0xa24450),
            Self::HoneyLavender => (false, 0xfff4dc, 0xfffcf7, 0xf1e3d5, 0x86562f),
            Self::PoppyRed => (true, 0x240c0c, 0x321515, 0x442020, 0xff9f94),
            Self::Mandarin => (true, 0x24150a, 0x332214, 0x46301f, 0xffbe7d),
            Self::MangoMilk => (false, 0xfff4dc, 0xfffdf6, 0xf4e6c8, 0x8b5112),
            Self::Sunflower => (true, 0x201d08, 0x2e2910, 0x3f381c, 0xf6d96c),
            Self::Daffodil => (false, 0xfffbe4, 0xfffef8, 0xf2eccb, 0x716013),
            Self::LimeWire => (true, 0x142009, 0x1f2e12, 0x2c3b1a, 0xd1f28a),
            Self::MatchaLatte => (false, 0xf1f8e6, 0xfcfff7, 0xe1edcf, 0x4a6e22),
            Self::RacingGreen => (true, 0x071d13, 0x102b1d, 0x1e3d2c, 0x83dba1),
            Self::Viridian => (true, 0x071e1b, 0x102d29, 0x1b3c36, 0x83e2cb),
            Self::AquaMarine => (false, 0xe8faf4, 0xf9fffd, 0xd3ede2, 0x226d5a),
            Self::CyanCircuit => (true, 0x061b22, 0x0e2933, 0x1b3b46, 0x73e6f4),
            Self::IceMint => (false, 0xe9fbfc, 0xf9ffff, 0xd7eef0, 0x1c6873),
            Self::RoyalNavy => (true, 0x080f25, 0x111b37, 0x1e2b4b, 0x93b9ff),
            Self::Cornflower => (false, 0xeef3ff, 0xfbfcff, 0xdde6f7, 0x3b5ea5),
            Self::FuchsiaPulse => (true, 0x240e26, 0x341a36, 0x472743, 0xff9fe3),
            Self::SakuraSilk => (false, 0xfff0f6, 0xfffafd, 0xf3deea, 0xa32f65),
            Self::Terracotta => (true, 0x241710, 0x34231a, 0x473327, 0xf1ba9b),
            Self::ToffeeCream => (false, 0xfbf2e6, 0xfffcf6, 0xeddfcc, 0x7e552e),
            Self::TrueBlack => (true, 0x050505, 0x101010, 0x1b1b1b, 0xd7d7d7),
            Self::SilverMist => (false, 0xf1f3f4, 0xfdfefe, 0xe0e5e8, 0x48565e),
            Self::PureWhite => (false, 0xffffff, 0xf8f8f8, 0xe7e7e7, 0x414141),
            Self::AshStone => (true, 0x181818, 0x222222, 0x303030, 0xc9c9c9),
            Self::LavenderInk => (true, 0x140d2a, 0x211734, 0x302344, 0xc7a4ff),
            Self::LilacPetal => (false, 0xf5eeff, 0xfdfaff, 0xe8dcf4, 0x7444aa),
            Self::RubySunrise => (true, 0x240c16, 0x2b1820, 0x3d2930, 0xffae9a),
            Self::MangoWave => (true, 0x25190c, 0x2d2217, 0x403222, 0xffcc8d),
            Self::LemonLime => (true, 0x24250a, 0x2a2c14, 0x353a20, 0xdaef8b),
            Self::JungleCurrent => (true, 0x0b2413, 0x132d21, 0x233e30, 0x9ce9a7),
            Self::CyanAurora => (true, 0x07242d, 0x102b36, 0x203c47, 0x80e2f3),
            Self::IndigoSpectrum => (true, 0x111436, 0x1b1e36, 0x2c3049, 0xb4b9ff),
            Self::FuchsiaLagoon => (true, 0x2b1028, 0x301c31, 0x422a43, 0xf8a0dc),
            Self::BronzeOcean => (true, 0x251c13, 0x2e241b, 0x40362a, 0xe4c297),
            Self::NoirPrism => (true, 0x090a0c, 0x161b22, 0x252e38, 0xc9d4e7),
            Self::CopperRose => (true, 0x261810, 0x2f211b, 0x423026, 0xf4bba4),
            Self::CitrusClouds => (false, 0xfff9dc, 0xfffdf7, 0xf2e7cf, 0x785713),
            Self::AquaBloom => (false, 0xe4f8ff, 0xfaffff, 0xdcecf1, 0x23647c),
            Self::BluePorcelain => (false, 0xe8f1ff, 0xfcfdff, 0xdde6f6, 0x3d60a2),
            Self::RoseGold => (false, 0xffe9f1, 0xfffaf8, 0xf2dfdf, 0xa13c58),
            Self::PistachioPeach => (false, 0xecf7df, 0xfefff8, 0xe5ecd3, 0x4d6b2a),
            Self::ArcticPearl => (false, 0xf9fcff, 0xffffff, 0xe2e9ef, 0x425b72),
            Self::RubySilk => (true, 0x250b15, 0x2b111a, 0x3c2028, 0xffa1ad),
            Self::ScarletEmber => (true, 0x230c09, 0x2d1510, 0x40251f, 0xffad93),
            Self::GarnetGlow => (true, 0x200a0d, 0x29111b, 0x3a1e28, 0xf5a4b7),
            Self::CherryLacquer => (true, 0x27080e, 0x32131a, 0x44202a, 0xff9ea9),
            Self::Rosefire => (true, 0x270c13, 0x30191f, 0x422831, 0xffb29f),
            Self::CrimsonTide => (true, 0x220a0b, 0x2b1419, 0x3e242b, 0xff9b9b),
            Self::VelvetRuby => (true, 0x1f0d10, 0x2a181d, 0x3c2930, 0xf3aab4),
            Self::BloodOrange => (true, 0x280c07, 0x321b13, 0x442c20, 0xffb18b),
            Self::StrawberryCream => (false, 0xffeceb, 0xfffaf8, 0xf0dbdb, 0x9e3345),
            Self::CherrySorbet => (false, 0xffe8eb, 0xfff9f7, 0xf0d9df, 0x9c2944),
            Self::RoseSatin => (false, 0xffeff0, 0xfffcfa, 0xf0e0df, 0x9b3a4f),
            Self::CoralVelvet => (false, 0xffeddf, 0xfffbf7, 0xf1dfd6, 0xa13930),
        };
        let rgb = |hex: u32| Color32::from_rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8);
        let (window, panel, surface, accent) = (rgb(window), rgb(panel), rgb(surface), rgb(accent));
        let text = if dark { rgb(0xf6f3ff) } else { rgb(0x21192e) };
        let mix = |a: Color32, b: Color32, amount: f32| {
            let channel = |a: u8, b: u8| {
                (f32::from(a) * (1.0 - amount) + f32::from(b) * amount).round() as u8
            };
            Color32::from_rgb(
                channel(a.r(), b.r()),
                channel(a.g(), b.g()),
                channel(a.b(), b.b()),
            )
        };
        Palette {
            dark,
            window,
            panel,
            surface,
            surface_hover: mix(surface, text, 0.08),
            surface_active: mix(surface, text, 0.14),
            outline: mix(surface, text, 0.17),
            text,
            secondary: mix(text, window, 0.24),
            dim: mix(text, window, 0.42),
            accent,
            accent_hover: mix(accent, text, 0.14),
            on_accent: if dark { rgb(0x160e22) } else { Color32::WHITE },
            danger: if dark { rgb(0xff8c9d) } else { rgb(0xb82845) },
            warning: if dark { rgb(0xf5cd7e) } else { rgb(0x805509) },
            overlay: panel,
            shadow: Color32::from_black_alpha(if dark { 150 } else { 40 }),
            tint: Color32::from_rgba_unmultiplied(
                accent.r(),
                accent.g(),
                accent.b(),
                if dark { 28 } else { 18 },
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn luminance(color: Color32) -> f32 {
        let channel = |value: u8| {
            let value = f32::from(value) / 255.0;
            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * channel(color.r()) + 0.7152 * channel(color.g()) + 0.0722 * channel(color.b())
    }

    fn contrast(a: Color32, b: Color32) -> f32 {
        let (a, b) = (luminance(a), luminance(b));
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    #[test]
    fn every_motion_theme_is_distinct_and_keeps_controls_readable() {
        for (index, theme) in MotionTheme::ALL.into_iter().enumerate() {
            let palette = theme.palette();
            assert!(
                contrast(palette.text, palette.window) >= 7.0,
                "{theme:?}: page text"
            );
            assert!(
                contrast(palette.text, palette.surface_active) >= 7.0,
                "{theme:?}: selected text"
            );
            assert!(
                contrast(palette.secondary, palette.panel) >= 4.5,
                "{theme:?}: secondary text"
            );
            assert!(
                contrast(palette.accent, palette.panel) >= 4.5,
                "{theme:?}: accent text"
            );
            assert!(
                contrast(palette.on_accent, palette.accent) >= 4.5,
                "{theme:?}: buttons"
            );
            assert!(
                contrast(palette.on_accent, palette.accent_hover) >= 4.5,
                "{theme:?}: hovered buttons"
            );
            assert!(MotionTheme::FAMILIES.contains(&theme.family()));
            for other in &MotionTheme::ALL[..index] {
                assert_ne!(theme.name(), other.name());
                assert_ne!(palette, other.palette());
            }
        }
    }

    #[test]
    fn themes_round_trip_with_readable_stable_names() {
        for theme in MotionTheme::ALL {
            let json = serde_json::to_string(&theme).unwrap();
            assert_eq!(serde_json::from_str::<MotionTheme>(&json).unwrap(), theme);
        }
        assert_eq!(
            serde_json::to_string(&MotionTheme::PurpleRain).unwrap(),
            r#""purple-rain""#
        );
        assert_eq!(
            MotionTheme::ALL
                .iter()
                .filter(|theme| !theme.palette().dark)
                .count(),
            55
        );
    }

    #[test]
    fn all_180_themes_have_unique_stable_identities_and_expected_collections() {
        let mut filenames = std::collections::BTreeSet::new();
        assert_eq!(MotionTheme::ALL.len(), 180);
        assert_eq!(MotionTheme::FAMILIES.len(), 12);
        for theme in MotionTheme::ALL {
            assert!(
                filenames.insert(theme.filename()),
                "{theme:?}: duplicate id"
            );
            assert_eq!(MotionTheme::from_filename(theme.filename()), Some(theme));
            let slug = theme.filename().strip_prefix("motion-theme:").unwrap();
            assert_eq!(serde_json::to_value(theme).unwrap().as_str(), Some(slug));
        }
        assert_eq!(MotionTheme::from_filename("Prism Night.json"), None);
        for (family, expected) in [
            ("Red", 12),
            ("Orange", 10),
            ("Yellow", 8),
            ("Green", 12),
            ("Teal", 6),
            ("Cyan", 7),
            ("Blue", 8),
            ("Purple", 20),
            ("Pink", 12),
            ("Brown", 8),
            ("Monochrome", 9),
            ("Gradients", 68),
        ] {
            assert_eq!(
                MotionTheme::ALL
                    .into_iter()
                    .filter(|theme| theme.family() == family)
                    .count(),
                expected,
                "{family}"
            );
        }
        // Existing defaults and representative saved spellings stay stable.
        assert_eq!(MotionTheme::Amethyst.filename(), "motion-theme:amethyst");
        assert_eq!(
            MotionTheme::LavenderMilk.filename(),
            "motion-theme:lavender-milk"
        );
        assert_eq!(
            Palette::dark().window,
            MotionTheme::Amethyst.palette().window
        );
        assert_eq!(
            Palette::light().window,
            MotionTheme::LavenderMilk.palette().window
        );
        assert!(MotionTheme::Amethyst.gradient().is_none());
        assert!(MotionTheme::LavenderMilk.gradient().is_none());
    }

    #[test]
    fn sixty_eight_gradients_keep_text_readable_across_the_whole_canvas() {
        let mut dark = 0;
        let mut light = 0;
        for theme in MotionTheme::ALL {
            let gradient = theme.gradient();
            assert_eq!(gradient.is_some(), theme.family() == "Gradients");
            let Some(stops) = gradient else { continue };
            let palette = theme.palette();
            if palette.dark {
                dark += 1
            } else {
                light += 1
            }
            assert_eq!(
                palette.window, stops[0],
                "{theme:?}: solid fallback matches canvas"
            );
            assert_ne!(stops[0], stops[1], "{theme:?}: first transition");
            assert_ne!(stops[1], stops[2], "{theme:?}: second transition");
            assert_ne!(stops[0], stops[2], "{theme:?}: separate endpoints");
            for pair in stops.windows(2) {
                for step in 0..=16 {
                    let amount = step as f32 / 16.0;
                    let blend = |a: u8, b: u8| {
                        (f32::from(a) * (1.0 - amount) + f32::from(b) * amount).round() as u8
                    };
                    let color = Color32::from_rgb(
                        blend(pair[0].r(), pair[1].r()),
                        blend(pair[0].g(), pair[1].g()),
                        blend(pair[0].b(), pair[1].b()),
                    );
                    assert!(
                        contrast(palette.text, color) >= 7.0,
                        "{theme:?}: canvas text"
                    );
                    assert!(
                        contrast(palette.secondary, color) >= 4.5,
                        "{theme:?}: canvas details"
                    );
                    assert_eq!(color.a(), 255);
                }
            }
        }
        assert_eq!((dark, light), (46, 22));
    }

    #[test]
    fn the_red_gradient_collection_keeps_every_canvas_stop_warm_and_searchable() {
        let themes = [
            MotionTheme::RubySilk,
            MotionTheme::ScarletEmber,
            MotionTheme::GarnetGlow,
            MotionTheme::CherryLacquer,
            MotionTheme::Rosefire,
            MotionTheme::CrimsonTide,
            MotionTheme::VelvetRuby,
            MotionTheme::BloodOrange,
            MotionTheme::StrawberryCream,
            MotionTheme::CherrySorbet,
            MotionTheme::RoseSatin,
            MotionTheme::CoralVelvet,
        ];
        let mut dark = 0;
        let mut light = 0;
        let mut distinct = std::collections::BTreeSet::new();
        for theme in themes {
            assert!(MotionTheme::ALL.contains(&theme));
            assert_eq!(theme.family(), "Gradients");
            let words: Vec<_> = theme.colour_keywords().split_whitespace().collect();
            assert!(words.contains(&"red") && words.contains(&"gradient"));
            let stops = theme.gradient().expect("a real red gradient");
            assert!(distinct.insert(stops.map(|stop| stop.to_array())));
            for stop in stops {
                assert!(
                    stop.r() > stop.g() && stop.r() > stop.b(),
                    "{theme:?}: each stop stays led by red"
                );
            }
            if theme.palette().dark {
                dark += 1;
                assert!(words.contains(&"dark"));
            } else {
                light += 1;
                assert!(words.contains(&"light"));
            }
        }
        assert_eq!((dark, light), (8, 4));
    }

    #[test]
    fn gradient_choices_and_system_pairs_restore_their_gradient_identity() {
        use crate::settings::ThemeChoice;
        use fastframe_theme::{Base, Pair};
        for theme in MotionTheme::ALL
            .into_iter()
            .filter(|theme| theme.gradient().is_some())
        {
            let choice = ThemeChoice::Motion(theme);
            let saved = serde_json::to_string(&choice).unwrap();
            assert_eq!(serde_json::from_str::<ThemeChoice>(&saved).unwrap(), choice);
            let custom = theme.as_custom_theme();
            assert_eq!(custom.palette, theme.palette());
            let base = Base::from_dark(custom.palette.dark);
            let mut pair = Pair::<Palette>::default();
            assert!(pair.set(base, Some(&custom)));
            assert!(!pair.set(Base::from_dark(!custom.palette.dark), Some(&custom)));
            let saved = serde_json::to_string(&pair).unwrap();
            let restored: Pair<Palette> = serde_json::from_str(&saved).unwrap();
            let identity = restored.filename(base).unwrap();
            let selected = MotionTheme::from_filename(identity).unwrap();
            assert_eq!(selected, theme);
            assert_eq!(selected.gradient(), theme.gradient());
            assert_eq!(restored.palette(base), theme.palette());
        }
    }
    #[test]
    fn each_colour_collection_has_light_and_dark_choices_and_useful_search_words() {
        for family in MotionTheme::FAMILIES {
            let choices: Vec<_> = MotionTheme::ALL
                .into_iter()
                .filter(|theme| theme.family() == family)
                .collect();
            assert!(
                choices.iter().any(|theme| theme.palette().dark),
                "{family}: a dark choice"
            );
            assert!(
                choices.iter().any(|theme| !theme.palette().dark),
                "{family}: a light choice"
            );
        }
        for (theme, needle) in [
            (MotionTheme::TrueBlack, "black"),
            (MotionTheme::PureWhite, "white"),
            (MotionTheme::Sunflower, "yellow"),
            (MotionTheme::RoyalNavy, "navy"),
            (MotionTheme::CyanCircuit, "cyan"),
            (MotionTheme::LimeWire, "lime"),
            (MotionTheme::ToffeeCream, "brown"),
            (MotionTheme::FuchsiaPulse, "fuchsia"),
            (MotionTheme::CyanAurora, "cyan"),
            (MotionTheme::RoseGold, "pink"),
        ] {
            assert!(
                theme
                    .colour_keywords()
                    .split_whitespace()
                    .any(|word| word == needle),
                "{theme:?}: found by {needle}"
            );
        }
        assert!(!MotionTheme::TrueBlack.colour_keywords().contains("white"));
        assert!(!MotionTheme::PureWhite.colour_keywords().contains("black"));
    }
}
