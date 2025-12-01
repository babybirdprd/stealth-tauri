import { useState, useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";

interface LogEntry {
  id: number;
  message: string;
  time: string;
}

export function NetworkTab() {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const endRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const unlisten = listen<string>("proxy://log", (event) => {
      setLogs(prev => [...prev.slice(-1000), { // Keep last 1000
          id: Date.now() + Math.random(),
          message: event.payload,
          time: new Date().toLocaleTimeString()
      }]);
    });
    return () => {
      unlisten.then(f => f());
    }
  }, []);

  useEffect(() => {
    endRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [logs]);

  return (
    <div className="flex-1 flex flex-col bg-gray-900 text-white h-full overflow-hidden">
      <div className="p-2 bg-gray-800 font-bold text-sm border-b border-gray-700">Network Logs</div>
      <div className="flex-1 overflow-y-auto p-4 font-mono text-xs">
          {logs.map(log => (
              <div key={log.id} className="mb-1 hover:bg-gray-800 p-1 rounded border-b border-gray-800 border-opacity-20">
                  <span className="text-gray-500 mr-2">[{log.time}]</span>
                  <span className={log.message.startsWith("REQ") ? "text-blue-400" : log.message.startsWith("RES") ? "text-green-400" : "text-gray-300"}>
                    {log.message}
                  </span>
              </div>
          ))}
          <div ref={endRef} />
      </div>
    </div>
  );
}
