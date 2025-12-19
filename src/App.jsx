import { useState, useEffect } from "react";
import { Store } from "@tauri-apps/plugin-store";
import "./App.css";

const store = new Store("settings.json");

function App() {
  const [apiKey, setApiKey] = useState("");
  const [preprompt, setPreprompt] = useState("Fix typos:");

  useEffect(() => {
    const loadSettings = async () => {
        try {
            const val = await store.get("api_key");
            if (val) setApiKey(val);
            const pp = await store.get("preprompt");
            if (pp) setPreprompt(pp);
        } catch (err) {
            console.error(err);
        }
    };
    loadSettings();
  }, []);

  const save = async () => {
    await store.set("api_key", apiKey);
    await store.set("preprompt", preprompt);
    await store.save();
    alert("Saved!");
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
      <button onClick={save}>Save Settings</button>
      <p className="hint">Select text and press <code>Alt+V</code> to fix.</p>
    </div>
  );
}

export default App;
