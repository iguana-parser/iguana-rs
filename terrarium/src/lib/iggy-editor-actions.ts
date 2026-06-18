import * as monaco from "monaco-editor";
import { commands } from "../bindings";

// Registers Terrarium's design-mode editor actions and keybinding overrides on
// the iggy grammar editor. The shared DesignView component registers none of its
// own, so the host attaches these through its onready callback. The actions
// dispatch window events the page listens for (open grammar, generate, parse,
// mode switches); Format calls the Tauri command and applies the result inline.
export function registerDesignEditorActions(editor: monaco.editor.IStandaloneCodeEditor) {
  editor.addAction({
    id: "terrarium.openGrammar",
    label: "Open Grammar",
    run: () => {
      window.dispatchEvent(new CustomEvent("terrarium-open-grammar"));
    },
  });
  editor.addAction({
    id: "terrarium.generate",
    label: "Generate Parser",
    keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyG],
    run: () => {
      window.dispatchEvent(new CustomEvent("terrarium-generate"));
    },
  });
  editor.addAction({
    id: "terrarium.parse",
    label: "Parse Input",
    keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyP],
    run: () => {
      window.dispatchEvent(new CustomEvent("terrarium-parse"));
    },
  });
  editor.addAction({
    id: "terrarium.mode.design",
    label: "Switch to Design Mode",
    keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyCode.Digit1],
    run: () => {
      window.dispatchEvent(new CustomEvent("terrarium-mode", { detail: "design" }));
    },
  });
  editor.addAction({
    id: "terrarium.mode.parse",
    label: "Switch to Parse Mode",
    keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyCode.Digit2],
    run: () => {
      window.dispatchEvent(new CustomEvent("terrarium-mode", { detail: "parse" }));
    },
  });
  editor.addAction({
    id: "terrarium.mode.debug",
    label: "Switch to Debug Mode",
    keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyCode.Digit3],
    run: () => {
      window.dispatchEvent(new CustomEvent("terrarium-mode", { detail: "debug" }));
    },
  });
  editor.addAction({
    id: "terrarium.formatGrammar",
    label: "Format Grammar",
    keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyMod.Shift | monaco.KeyCode.KeyF],
    run: async () => {
      const model = editor.getModel();
      if (!model) return;
      const source = model.getValue();
      const formatted = await commands.formatGrammar(source);
      if (formatted === null || formatted === source) return;
      // Uses executeEdits (not setValue) to avoid resetting semantic tokens,
      // which would cause a white flash while tokens are re-fetched.
      editor.executeEdits("format", [{ range: model.getFullModelRange(), text: formatted }]);
    },
  });

  // Declarative remapping for built-in Monaco actions that conflict with
  // Terrarium shortcuts.
  monaco.editor.addKeybindingRules([
    { keybinding: monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyP, command: "-editor.action.quickCommand" },
    { keybinding: monaco.KeyCode.F3, command: "-editor.action.nextMatchFindAction" },
    // F3: Go to Definition
    { keybinding: monaco.KeyCode.F3, command: "editor.action.revealDefinition" },
    // Cmd+O: Quick Outline
    { keybinding: monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyO, command: "editor.action.quickOutline" },
    // Cmd+Shift+P: Command palette
    { keybinding: monaco.KeyMod.CtrlCmd | monaco.KeyMod.Shift | monaco.KeyCode.KeyP, command: "editor.action.quickCommand" },
    // Cmd+D: Delete line
    { keybinding: monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyD, command: "editor.action.deleteLines" },
    // Cmd+[/]: Cursor back/forward (unbind indent/outdent)
    { keybinding: monaco.KeyMod.CtrlCmd | monaco.KeyCode.BracketLeft, command: "-editor.action.outdentLines" },
    { keybinding: monaco.KeyMod.CtrlCmd | monaco.KeyCode.BracketRight, command: "-editor.action.indentLines" },
    { keybinding: monaco.KeyMod.CtrlCmd | monaco.KeyCode.BracketLeft, command: "cursorUndo" },
    { keybinding: monaco.KeyMod.CtrlCmd | monaco.KeyCode.BracketRight, command: "cursorRedo" },
    // Cmd+.: Jump to next error/warning marker (unbind Quick Fix default)
    { keybinding: monaco.KeyMod.CtrlCmd | monaco.KeyCode.Period, command: "-editor.action.quickFix" },
    { keybinding: monaco.KeyMod.CtrlCmd | monaco.KeyCode.Period, command: "editor.action.marker.next" },
    // Cmd+L: Go to line (unbind expand-line-selection default)
    { keybinding: monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyL, command: "-expandLineSelection" },
    { keybinding: monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyL, command: "editor.action.gotoLine" },
  ]);
}
