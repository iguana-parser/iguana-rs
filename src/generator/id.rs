use std::collections::HashMap;

use indexmap::{IndexMap, IndexSet};

use crate::{
    grammar::symbols::{Nonterminal, Terminal},
    ids::{NonterminalId, SlotId, TerminalId},
};

#[derive(Debug)]
pub struct EndSlot {
    pub slot_id: SlotId,
    pub index: usize,
}

#[derive(Default)]
pub struct NonterminalIds {
    // nonterminals[i] = the nonterminal with id i
    nonterminals: IndexSet<Nonterminal>,
    // Indexed by nonterminal ids to a list of their end grammar slots.
    // end_slots[nonterminal_id] = end slots for the nonterminal's alternatives.
    alternatives: IndexMap<NonterminalId, Vec<EndSlot>>,
}

impl NonterminalIds {
    pub fn insert(&mut self, nonterminal: Nonterminal) {
        self.nonterminals.insert(nonterminal);
    }
    pub fn get_id(&self, nonterminal: &Nonterminal) -> Option<NonterminalId> {
        let id = self.nonterminals.get_index_of(nonterminal);
        id.map(|id| NonterminalId(id as u16))
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
    pub fn add_end_slot(&mut self, nonterminal_id: NonterminalId, alternative: EndSlot) {
        self.alternatives
            .entry(nonterminal_id)
            .or_default()
            .push(alternative);
    }
    pub fn end_slots(&self, nonterminal_id: NonterminalId) -> impl Iterator<Item = &EndSlot> {
        self.alternatives[nonterminal_id.index()].iter()
    }
    pub fn get_nonterminal(&self, nonterminal_id: NonterminalId) -> &Nonterminal {
        &self.nonterminals[nonterminal_id.index()]
    }
}

#[derive(Default)]
pub struct SlotIds {
    value: usize,
    slot_to_id: HashMap<String, usize>,
    slots: Vec<String>,
}

impl SlotIds {
    pub fn id(&mut self, name: &str) -> SlotId {
        if let Some(id) = self.slot_to_id.get(name) {
            SlotId(*id as u16)
        } else {
            let value = self.value;
            self.value += 1;
            self.slot_to_id.insert(name.to_owned(), value);
            self.slots.push(name.to_owned());
            SlotId(value as u16)
        }
    }
    pub fn len(&self) -> usize {
        self.slots.len()
    }
    pub fn slot_name(&self, slot_id: &SlotId) -> &str {
        &self.slots[slot_id.index()]
    }
    pub fn slots(&self) -> impl Iterator<Item = &str> {
        self.slots.iter().map(|s| s.as_str())
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
    pub fn get_id(&self, terminal: &Terminal) -> Option<TerminalId> {
        let id = self.terminals.get_index_of(terminal);
        id.map(|id| TerminalId(id as u16))
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
