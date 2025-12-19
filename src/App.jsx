import { useState, useEffect } from "react";
import { load } from "@tauri-apps/plugin-store";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "./components/ui/card";
import { Input } from "./components/ui/input";
import { Label } from "./components/ui/label";
import { Button } from "./components/ui/button";
import { Textarea } from "./components/ui/textarea";
import { Switch } from "./components/ui/switch";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "./components/ui/select";
import { Separator } from "./components/ui/separator";
import { Badge } from "./components/ui/badge";
import { Sparkles, Zap, Bell, Save, Check, AlertCircle, Wand2, KeyRound } from "lucide-react";
import "./App.css";

function App() {
  const [apiKey, setApiKey] = useState("");
  const [preprompt, setPreprompt] = useState("Fix typos:");
  const [turboMode, setTurboMode] = useState(false);
  const [selectedModel, setSelectedModel] = useState("gemini-2.5-flash");
  const [models, setModels] = useState([]);
  const [showDuck, setShowDuck] = useState(true);
  const [showNotification, setShowNotification] = useState(false);

  const [store, setStore] = useState(null);
  const [modelsLoading, setModelsLoading] = useState(false);
  const [saveStatus, setSaveStatus] = useState("idle"); // idle, saving, success, error

  // Fetch models function
  const fetchModels = async (key) => {
    if (!key || key.length < 10) return;
    setModelsLoading(true);
    try {
      const result = await invoke("fetch_models", { apiKey: key });
      setModels(result);
    } catch (err) {
      console.error("Failed to fetch models:", err);
    } finally {
      setModelsLoading(false);
    }
  };

  // Load settings
  useEffect(() => {
    const loadSettings = async () => {
      try {
        const _store = await load("settings.json", { autoSave: false });
        setStore(_store);

        const val = await _store.get("api_key");
        if (val) setApiKey(val);

        const pp = await _store.get("preprompt");
        if (pp) setPreprompt(pp);

        const tm = await _store.get("turbo_mode");
        if (tm !== undefined) setTurboMode(tm);

        const model = await _store.get("model");
        if (model) setSelectedModel(model);

        const duck = await _store.get("show_duck");
        if (duck !== undefined) setShowDuck(duck);

        const notif = await _store.get("show_notification");
        if (notif !== undefined) setShowNotification(notif);

        // Auto-fetch models if we have an API key
        if (val) {
          fetchModels(val);
        }
      } catch (err) {
        console.error("Failed to load settings:", err);
      }
    };
    loadSettings();
  }, []);

  // Auto-fetch models when API key changes (debounced)
  useEffect(() => {
    if (apiKey && apiKey.length >= 10) {
      const debounce = setTimeout(() => {
        fetchModels(apiKey);
      }, 800);
      return () => clearTimeout(debounce);
    }
  }, [apiKey]);

  // Auto-clear save status
  useEffect(() => {
    if (saveStatus === "success" || saveStatus === "error") {
      const timer = setTimeout(() => setSaveStatus("idle"), 2000);
      return () => clearTimeout(timer);
    }
  }, [saveStatus]);

  // Save settings
  const save = async () => {
    if (!store) {
      console.warn("Store not ready");
      return;
    }
    setSaveStatus("saving");
    try {
      await store.set("api_key", apiKey);
      await store.set("preprompt", preprompt);
      await store.set("turbo_mode", turboMode);
      await store.set("model", selectedModel);
      await store.set("show_duck", showDuck);
      await store.set("show_notification", showNotification);
      await store.save();
      setSaveStatus("success");
    } catch (e) {
      console.error("Save failed:", e);
      setSaveStatus("error");
    }
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-purple-50 via-blue-50 to-pink-50 pb-24" style={{ fontFamily: 'var(--font-family-body)' }}>
      <div className="max-w-3xl mx-auto p-8 space-y-6">
        {/* Header */}
        <div className="text-center space-y-2">
          <div className="flex items-center justify-center gap-2">
            <h1 className="text-4xl" style={{ fontFamily: 'var(--font-family-heading)', fontWeight: 700, letterSpacing: '-0.02em' }}>FixTypo Settings</h1>
          </div>
          <div className="flex justify-center mt-2">
            <Badge variant="secondary" className="gap-1">
              Press <kbd className="px-2 py-1 bg-background rounded border font-mono">Ctrl+Q</kbd> to fix selected text
            </Badge>
          </div>
        </div>

        {/* API Configuration */}
        <Card>
          <CardHeader>
            <div className="flex items-center gap-2">
              <KeyRound className="w-5 h-5 text-purple-600" />
              <CardTitle>API Configuration</CardTitle>
            </div>
            <CardDescription>
              Connect to Gemini AI to power your typo-fixing magic
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="api-key">Gemini API Key</Label>
              <Input
                id="api-key"
                type="password"
                value={apiKey}
                onChange={(e) => setApiKey(e.target.value)}
                placeholder="Enter your Gemini API key..."
              />
              <p className="text-xs text-muted-foreground">
                Don't have an API key?{" "}
                <a
                  href="https://aistudio.google.com/app/apikey"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-purple-600 hover:underline"
                >
                  Get one here
                </a>
              </p>
            </div>

            <div className="space-y-2">
              <Label htmlFor="model">AI Model {modelsLoading && <span className="text-xs text-muted-foreground ml-2">(Fetching...)</span>}</Label>
              <Select value={selectedModel} onValueChange={setSelectedModel}>
                <SelectTrigger id="model">
                  <SelectValue placeholder="Select a model" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="gemini-2.5-flash">gemini-2.5-flash</SelectItem>
                  {models.filter(m => m !== "gemini-2.5-flash").map(m => (
                    <SelectItem key={m} value={m}>{m}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </CardContent>
        </Card>

        {/* Behavior Settings */}
        <Card>
          <CardHeader>
            <div className="flex items-center gap-2">
              <Wand2 className="w-5 h-5 text-purple-600" />
              <CardTitle>Behavior</CardTitle>
            </div>
            <CardDescription>
              Customize how FixTypo works for you
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-6">
            <div className="space-y-2">
              <Label htmlFor="preprompt">Custom Instructions</Label>
              <Textarea
                id="preprompt"
                value={preprompt}
                onChange={(e) => setPreprompt(e.target.value)}
                placeholder="E.g., Fix typos and make it sound professional..."
                rows={4}
                className="resize-none"
              />
              <p className="text-sm text-muted-foreground">
                Tell the AI how you want your text corrected
              </p>
            </div>

            <Separator />

            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <Zap className="w-5 h-5 text-yellow-600" />
                <div className="space-y-0.5">
                  <Label htmlFor="turbo-mode">Turbo Mode</Label>
                  <p className="text-sm text-muted-foreground">
                    Automatically copy fixed text to clipboard and past result
                  </p>
                </div>
              </div>
              <Switch
                id="turbo-mode"
                checked={turboMode}
                onCheckedChange={setTurboMode}
              />
            </div>
          </CardContent>
        </Card>

        {/* Interface Settings */}
        <Card>
          <CardHeader>
            <div className="flex items-center gap-2">
              <Sparkles className="w-5 h-5 text-purple-600" />
              <CardTitle>Interface</CardTitle>
            </div>
            <CardDescription>
              Make FixTypo your own with these fun options
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className="w-5 h-5 text-xl flex items-center justify-center">🦆</div>
                <div className="space-y-0.5">
                  <Label htmlFor="show-duck">Duck Animation</Label>
                  <p className="text-sm text-muted-foreground">
                    Show a cute duck while processing
                  </p>
                </div>
              </div>
              <Switch
                id="show-duck"
                checked={showDuck}
                onCheckedChange={setShowDuck}
              />
            </div>

            <Separator />

            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <Bell className="w-5 h-5 text-blue-600" />
                <div className="space-y-0.5">
                  <Label htmlFor="show-notification">Notifications</Label>
                  <p className="text-sm text-muted-foreground">
                    Get notified when your text is ready
                  </p>
                </div>
              </div>
              <Switch
                id="show-notification"
                checked={showNotification}
                onCheckedChange={setShowNotification}
              />
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Sticky Save Button */}
      <div className="fixed bottom-0 left-0 right-0 bg-gradient-to-t from-white via-white to-transparent p-4">
        <div className="max-w-3xl mx-auto">
          <Button
            onClick={save}
            disabled={!store || saveStatus === "saving"}
            className="w-full h-12 text-lg shadow-lg"
            variant={saveStatus === "success" ? "default" : saveStatus === "error" ? "destructive" : "default"}
          >
            {saveStatus === "saving" && (
              <>
                <Save className="w-5 h-5 mr-2 animate-pulse" />
                Saving...
              </>
            )}
            {saveStatus === "success" && (
              <>
                <Check className="w-5 h-5 mr-2" />
                Saved Successfully!
              </>
            )}
            {saveStatus === "error" && (
              <>
                <AlertCircle className="w-5 h-5 mr-2" />
                Error Saving
              </>
            )}
            {saveStatus === "idle" && (
              <>
                <Save className="w-5 h-5 mr-2" />
                Save Settings
              </>
            )}
          </Button>
        </div>
      </div>
    </div>
  );
}

export default App;
