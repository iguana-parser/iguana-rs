use std::collections::HashMap;

use indexmap::{IndexMap, IndexSet};
use itertools::Itertools;

use crate::{
    grammar::{
        def::Grammar,
        regex::{CharClass, Regex},
        slot::Slot,
        symbols::{Nonterminal, Terminal},
    },
    ids::{CharClassId, NonterminalId, SlotId, TerminalId},
};

#[derive(Debug)]
pub struct EndSlot {
    pub slot_id: SlotId,
    pub index: usize,
}

pub struct NonterminalIds {
    // nonterminals[i] = the nonterminal with id i
    nonterminals: IndexSet<Nonterminal>,
    // Indexed by nonterminal ids to a list of their end grammar slots.
    // end_slots[nonterminal_id] = end slots for the nonterminal's alternatives.
    alternatives: IndexMap<NonterminalId, Vec<EndSlot>>,
    // The id of the first data-dependent nonterminal.
    dd_id_start: usize,
}

impl NonterminalIds {
    pub fn new(nonterminals: impl Iterator<Item = Nonterminal>) -> Self {
        // We sort the nonterminals so that nonterminals without parameters come first.
        // This is because we use a single vector `gss_nodes_index` for such nonterminals.
        // For data-dependent nonterminals which have parameters, we generate a separate
        let nonterminals: IndexSet<_> = nonterminals
            .sorted_by_key(|nt| !nt.parameters.is_empty())
            .collect();
        let dd_id_start = nonterminals
            .iter()
            .position(|nt| !nt.parameters.is_empty())
            .unwrap_or(nonterminals.len());
        Self {
            nonterminals,
            alternatives: IndexMap::default(),
            dd_id_start,
        }
    }

    pub fn get_id(&self, nonterminal: &Nonterminal) -> NonterminalId {
        let id = self.nonterminals.get_index_of(nonterminal);
        id.map(|id| NonterminalId(id as u16))
            .unwrap_or_else(|| panic!("unknown nonterminal: {}", nonterminal))
    }
    pub fn len(&self) -> usize {
        self.nonterminals.len()
    }
    pub fn ids(&self) -> impl Iterator<Item = NonterminalId> {
        (0..self.len()).map(|id| NonterminalId(id as u16))
    }
    pub fn nonterminals(&self) -> impl Iterator<Item = &Nonterminal> {
        self.nonterminals.iter()
    }
    pub fn dd_nonterminals(&self) -> impl Iterator<Item = &Nonterminal> {
        self.nonterminals.iter().skip(self.dd_id_start)
    }
    pub fn add_end_slot(&mut self, nonterminal_id: NonterminalId, alternative: EndSlot) {
        self.alternatives
            .entry(nonterminal_id)
            .or_default()
            .push(alternative);
    }
    pub fn end_slots(&self, nonterminal_id: NonterminalId) -> impl Iterator<Item = &EndSlot> {
        self.alternatives.get(&nonterminal_id).unwrap().iter()
    }
    pub fn get_nonterminal(&self, nonterminal_id: NonterminalId) -> &Nonterminal {
        &self.nonterminals[nonterminal_id.index()]
    }
}

pub struct SlotIds<'a> {
    grammar: &'a Grammar,
    value: usize,
    slot_to_id: HashMap<Slot<'a>, usize>,
    slots: Vec<Slot<'a>>,
}

impl<'a> SlotIds<'a> {
    pub fn new(grammar: &'a Grammar) -> Self {
        Self {
            grammar,
            value: 0,
            slot_to_id: HashMap::new(),
            slots: vec![],
        }
    }
    pub fn insert(&mut self, slot: Slot<'a>) -> SlotId {
        let value = self.value;
        self.value += 1;
        self.slot_to_id.insert(slot.clone(), value);
        self.slots.push(slot);
        SlotId(value as u16)
    }
    pub fn get_id(&self, slot: &Slot<'a>) -> SlotId {
        self.slot_to_id
            .get(slot)
            .map(|id| SlotId(*id as u16))
            .unwrap_or_else(|| panic!("unknown slot: {}", slot.name()))
    }
    pub fn len(&self) -> usize {
        self.slots.len()
    }
    pub fn display_name(&self, slot_id: &SlotId) -> String {
        self.slots[slot_id.index()].display_name(self.grammar)
    }
    pub fn slots(&self) -> impl Iterator<Item = &Slot<'a>> {
        self.slots.iter()
    }
}

#[derive(Debug, Default)]
pub struct TerminalIds {
    // terminals[i] = the terminal with id i
    terminals: IndexSet<Terminal>,
}

impl TerminalIds {
    pub fn insert(&mut self, terminal: Terminal) {
        self.terminals.insert(terminal);
    }
    pub fn get_id(&self, terminal: &Terminal) -> TerminalId {
        let id = self.terminals.get_index_of(terminal);
        id.map(|id| TerminalId(id as u16))
            .unwrap_or_else(|| panic!("unknown terminal: {}", terminal))
    }
    pub fn ids(&self) -> impl Iterator<Item = TerminalId> {
        (0..self.len()).map(|id| TerminalId(id as u16))
    }
    pub fn len(&self) -> usize {
        self.terminals.len()
    }
    pub fn terminals(&self) -> impl Iterator<Item = &Terminal> {
        self.terminals.iter()
    }
}

#[derive(Debug, Default)]
pub struct CharClassIds {
    char_classes: IndexSet<CharClass>,
}

impl CharClassIds {
    pub fn insert(&mut self, char_class: CharClass) {
        self.char_classes.insert(char_class);
    }

    pub fn get_id(&self, char_class: &CharClass) -> Option<CharClassId> {
        self.char_classes
            .get_index_of(char_class)
            .map(|id| CharClassId(id as u16))
    }

    pub fn ids(&self) -> impl Iterator<Item = CharClassId> {
        (0..self.len()).map(|id| CharClassId(id as u16))
    }

    pub fn len(&self) -> usize {
        self.char_classes.len()
    }

    pub fn char_classes(&self) -> impl Iterator<Item = &CharClass> {
        self.char_classes.iter()
    }

    pub fn get(&self, id: CharClassId) -> &CharClass {
        &self.char_classes[id.index()]
    }
}

/// Collects all character classes from a regex into the CharClassIds set.
pub fn collect_char_classes(regex: &Regex, char_class_ids: &mut CharClassIds) {
    match regex {
        Regex::CharClass(cc) => {
            char_class_ids.insert(cc.clone());
        }
        Regex::Seq(rs) | Regex::Alt(rs) => {
            for r in rs {
                collect_char_classes(r, char_class_ids);
            }
        }
        Regex::Star(r) | Regex::Plus(r) | Regex::Opt(r) => {
            collect_char_classes(r, char_class_ids);
        }
        Regex::Char(_) | Regex::CharRange(_) | Regex::Epsilon => {}
        Regex::Identifier(_) => {
            unreachable!("Regex::Identifier should be inlined before code generation")
        }
    }
}
