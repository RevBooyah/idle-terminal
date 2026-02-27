use crate::game::buildings::BuildingKind;
use crate::game::upgrades::UpgradeId;
use crate::layout::PaneId;

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Render,
    GameTick,
    Quit,
    Resize(u16, u16),
    NextPane,
    PrevPane,
    FocusPane(PaneId),

    // Building actions
    PurchaseBuilding(BuildingKind),
    UpgradeBuilding(BuildingKind),

    // Upgrade actions
    PurchaseUpgrade(UpgradeId),

    // Task actions
    TaskInput(char),
    TaskSelect(usize),
    TaskSubmit,

    // Prestige
    Prestige,

    None,
}
