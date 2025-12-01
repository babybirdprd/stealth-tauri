import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

interface Profile {
  name: string;
  user_agent: string;
}

function App() {
  const [profiles, setProfiles] = useState<Profile[]>([]);
  const [selectedProfile, setSelectedProfile] = useState<string>("");
  const [scripts, setScripts] = useState<string[]>([]);
  const [scriptName, setScriptName] = useState<string>("untitled.rhai");
  const [scriptContent, setScriptContent] = useState<string>("");
  const [logs, setLogs] = useState<string[]>([]);

  const logsEndRef = useRef<HTMLDivElement>(null);

  const scrollToBottom = () => {
    logsEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }

  useEffect(() => {
    scrollToBottom();
  }, [logs]);

  useEffect(() => {
    // Initial data fetch
    invoke<Profile[]>("get_profiles").then(p => {
        setProfiles(p);
        if (p.length > 0) setSelectedProfile(p[0].name);
    });

    refreshScripts();

    // Listen for logs
    const unlisten = listen<string>("log_output", (event) => {
        setLogs(prev => [...prev, event.payload]);
    });

    return () => {
        unlisten.then(f => f());
    }
  }, []);

  const refreshScripts = () => {
      invoke<string[]>("list_scripts").then(setScripts);
  };

  const runScript = async () => {
    setLogs(prev => [...prev, `> Running script...`]);
    try {
        await invoke("execute_script", { script: scriptContent });
    } catch (e) {
        setLogs(prev => [...prev, `Error starting script: ${e}`]);
    }
  };

  const saveScript = async () => {
      if (!scriptName) return;
      try {
          await invoke("save_script", { filename: scriptName, content: scriptContent });
          setLogs(prev => [...prev, `Saved ${scriptName}`]);
          refreshScripts();
      } catch (e) {
          setLogs(prev => [...prev, `Error saving: ${e}`]);
      }
  };

  const loadScript = async (name: string) => {
      try {
          const content = await invoke<string>("read_script", { filename: name });
          setScriptContent(content);
          setScriptName(name);
          setLogs(prev => [...prev, `Loaded ${name}`]);
      } catch (e) {
          setLogs(prev => [...prev, `Error loading ${name}: ${e}`]);
      }
  };

  const handleProfileChange = (name: string) => {
      setSelectedProfile(name);
      invoke("set_profile", { profileName: name });
      setLogs(prev => [...prev, `Switched profile to ${name}`]);
  };

  return (
    <div className="flex h-screen w-screen bg-gray-900 text-white overflow-hidden font-mono">
        {/* Sidebar */}
        <div className="w-64 bg-gray-800 border-r border-gray-700 flex flex-col">
            <div className="p-4 font-bold text-lg bg-gray-900">Phantom Browser</div>
            <div className="flex-1 overflow-y-auto p-2">
                <div className="text-xs text-gray-500 mb-2 uppercase">Scripts</div>
                {scripts.map(s => (
                    <div key={s}
                         className="cursor-pointer hover:bg-gray-700 p-2 rounded text-sm truncate"
                         onClick={() => loadScript(s)}
                    >
                        {s}
                    </div>
                ))}
            </div>
        </div>

        {/* Main Content */}
        <div className="flex-1 flex flex-col">
            {/* Top Bar */}
            <div className="h-14 bg-gray-800 border-b border-gray-700 flex items-center px-4 justify-between">
                <div className="flex items-center space-x-4">
                    <div className="flex flex-col">
                        <label className="text-[10px] text-gray-400">IDENTITY</label>
                        <select
                            className="bg-gray-700 border-none rounded p-1 text-sm outline-none"
                            value={selectedProfile}
                            onChange={(e) => handleProfileChange(e.target.value)}
                        >
                            {profiles.map(p => (
                                <option key={p.name} value={p.name}>{p.name}</option>
                            ))}
                        </select>
                    </div>

                    <div className="flex flex-col flex-1 min-w-[200px]">
                        <label className="text-[10px] text-gray-400">SCRIPT NAME</label>
                        <input
                            className="bg-gray-900 p-1 rounded border border-gray-600 text-sm w-full"
                            value={scriptName}
                            onChange={(e) => setScriptName(e.target.value)}
                        />
                    </div>
                </div>

                <div className="flex space-x-2">
                    <button onClick={saveScript} className="px-3 py-1 bg-blue-600 hover:bg-blue-500 rounded text-sm">Save</button>
                    <button onClick={runScript} className="px-3 py-1 bg-green-600 hover:bg-green-500 rounded text-sm font-bold">RUN</button>
                </div>
            </div>

            {/* Editor */}
            <div className="flex-1 relative">
                <textarea
                    className="w-full h-full bg-gray-900 p-4 outline-none resize-none font-mono text-sm"
                    value={scriptContent}
                    onChange={(e) => setScriptContent(e.target.value)}
                    placeholder="// Type Rhai script here... e.g. browser.navigate('https://example.com')"
                    spellCheck={false}
                />
            </div>

            {/* Console */}
            <div className="h-48 bg-black border-t border-gray-700 flex flex-col">
                 <div className="bg-gray-800 px-2 py-1 text-xs text-gray-400">Console Output</div>
                 <div className="flex-1 overflow-y-auto p-2 font-mono text-xs text-green-400">
                    {logs.map((log, i) => (
                        <div key={i}>{log}</div>
                    ))}
                    <div ref={logsEndRef} />
                 </div>
            </div>
        </div>
    </div>
  );
}

export default App;
