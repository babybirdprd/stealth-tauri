import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface ProxyConfig {
    protocol: string;
    host: string;
    port: number;
    username?: string;
    password?: string;
}

export interface Profile {
    name: string;
    user_agent: string;
    seed: number;
    proxy?: ProxyConfig;
}

interface Props {
    profile: Profile | null;
    onUpdate: (p: Profile) => void;
}

export function SettingsTab({ profile, onUpdate }: Props) {
    const [localProfile, setLocalProfile] = useState<Profile | null>(null);

    useEffect(() => {
        if (profile) {
            setLocalProfile(JSON.parse(JSON.stringify(profile))); // Deep copy
        }
    }, [profile]);

    const handleSave = async () => {
        if (!localProfile) return;
        try {
             await invoke("save_profile_config", { profile: localProfile });
             onUpdate(localProfile);
             alert("Settings saved and proxy restarted.");
        } catch (e) {
            alert("Error: " + e);
        }
    };

    if (!localProfile) return <div className="p-4">No profile selected</div>;

    const updateProxy = (field: keyof ProxyConfig, value: any) => {
        setLocalProfile(prev => {
            if (!prev) return null;
            const newProxy = { ...(prev.proxy || { protocol: "http", host: "", port: 8080, username: "", password: "" }), [field]: value };
            return { ...prev, proxy: newProxy };
        });
    };

    return (
        <div className="flex-1 p-6 bg-gray-900 text-white overflow-y-auto">
            <h2 className="text-xl font-bold mb-4">Profile Settings: {localProfile.name}</h2>

            <div className="mb-6">
                <label className="block text-sm text-gray-400 mb-1">User Agent</label>
                <textarea
                    className="w-full bg-gray-800 p-2 rounded border border-gray-700 text-xs"
                    rows={3}
                    value={localProfile.user_agent}
                    onChange={e => setLocalProfile({...localProfile, user_agent: e.target.value})}
                />
            </div>

            <div className="mb-6">
                <label className="block text-sm text-gray-400 mb-1">Profile Seed (Fingerprint)</label>
                <input
                    type="number"
                    className="w-full bg-gray-800 p-2 rounded border border-gray-700 text-sm"
                    value={localProfile.seed}
                    onChange={e => setLocalProfile({...localProfile, seed: parseInt(e.target.value) || 0})}
                />
            </div>

            <div className="border-t border-gray-700 pt-4">
                <h3 className="text-lg font-bold mb-4 text-blue-400">Upstream Proxy</h3>

                <div className="grid grid-cols-2 gap-4 mb-4">
                    <div>
                        <label className="block text-xs text-gray-500 mb-1">Protocol</label>
                        <select
                            className="w-full bg-gray-800 p-2 rounded border border-gray-700 text-sm"
                            value={localProfile.proxy?.protocol || "http"}
                            onChange={e => updateProxy("protocol", e.target.value)}
                        >
                            <option value="http">HTTP</option>
                            <option value="socks5">SOCKS5</option>
                        </select>
                    </div>
                    <div>
                        <label className="block text-xs text-gray-500 mb-1">Port</label>
                        <input
                            type="number"
                            className="w-full bg-gray-800 p-2 rounded border border-gray-700 text-sm"
                            value={localProfile.proxy?.port || 8080}
                            onChange={e => updateProxy("port", parseInt(e.target.value) || 0)}
                        />
                    </div>
                </div>

                <div className="mb-4">
                    <label className="block text-xs text-gray-500 mb-1">Host</label>
                    <input
                        className="w-full bg-gray-800 p-2 rounded border border-gray-700 text-sm"
                        placeholder="e.g. proxy.example.com"
                        value={localProfile.proxy?.host || ""}
                        onChange={e => updateProxy("host", e.target.value)}
                    />
                </div>

                <div className="grid grid-cols-2 gap-4 mb-4">
                    <div>
                        <label className="block text-xs text-gray-500 mb-1">Username (Optional)</label>
                        <input
                            className="w-full bg-gray-800 p-2 rounded border border-gray-700 text-sm"
                            value={localProfile.proxy?.username || ""}
                            onChange={e => updateProxy("username", e.target.value)}
                        />
                    </div>
                    <div>
                        <label className="block text-xs text-gray-500 mb-1">Password (Optional)</label>
                        <input
                            type="password"
                            className="w-full bg-gray-800 p-2 rounded border border-gray-700 text-sm"
                            value={localProfile.proxy?.password || ""}
                            onChange={e => updateProxy("password", e.target.value)}
                        />
                    </div>
                </div>
            </div>

            <button
                onClick={handleSave}
                className="mt-4 px-4 py-2 bg-blue-600 hover:bg-blue-500 rounded text-sm font-bold w-full"
            >
                Save & Restart Proxy
            </button>
        </div>
    );
}
