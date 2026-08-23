use crate::widgets::ComboPart;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbeddedFocus {
    Combo { field_index: usize, part: ComboPart },
    Text { field_index: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionFieldAction {
    Field(usize),
    ComboPart { field: usize, part: ComboPart },
}
