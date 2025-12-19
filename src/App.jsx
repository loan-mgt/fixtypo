import { useState, useEffect } from "react";
import { load } from "@tauri-apps/plugin-store";
import "./App.css";

function App() {
  const [apiKey, setApiKey] = useState("");
  const [preprompt, setPreprompt] = useState("Fix typos:");
  const [turboMode, setTurboMode] = useState(false);
  const [store, setStore] = useState(null);

  useEffect(() => {
    const loadSettings = async () => {
      try {
        console.log("Loading store...");
        const _store = await load("settings.json", { autoSave: false });
        setStore(_store);
        console.log("Store loaded successfully");

        const val = await _store.get("api_key");
        if (val) setApiKey(val);
        const pp = await _store.get("preprompt");
        if (pp) setPreprompt(pp);
        const tm = await _store.get("turbo_mode");
        if (tm !== undefined) setTurboMode(tm);
      } catch (err) {
        console.error("Failed to load settings:", err);
        alert("Failed to load store: " + err);
      }
    };
    loadSettings();
  }, []);

  const save = async () => {
    if (!store) {
      alert("Store is not ready yet. Please wait or restart the app.");
      return;
    }
    try {
      await store.set("api_key", apiKey);
      await store.set("preprompt", preprompt);
      await store.set("turbo_mode", turboMode);
      await store.save();
      alert("Saved!");
    } catch (e) {
      console.error("Save failed:", e);
      alert("Save failed: " + e);
    }
  };

  return (
    <div className="container">
      <h3>AI Shortcut Config</h3>
      <div className="row">
        <input
          value={apiKey}
          onChange={e => setApiKey(e.target.value)}
          placeholder="Gemini API Key"
          className="input"
          type="password"
        />
      </div>
      <div className="row">
        <label>Pre-prompt:</label>
        <textarea
          value={preprompt}
          onChange={e => setPreprompt(e.target.value)}
          className="textarea"
          rows={5}
        />
      </div>
      <div className="row checkbox-row">
        <label>
          <input
            type="checkbox"
            checked={turboMode}
            onChange={e => setTurboMode(e.target.checked)}
          />
          Turbo Mode (auto copy/paste selected text)
        </label>
      </div>
      <button onClick={save} disabled={!store}>Save Settings</button>
      <p className="hint">Select text and press <code>Ctrl+Q</code> to fix.</p>
    </div>
  );
}

export default App;
