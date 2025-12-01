import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface Job {
    id: string;
    script_path: string;
    cron: string;
    profile?: string;
    last_run?: string;
    status: string;
}

export function JobsTab() {
    const [jobs, setJobs] = useState<Job[]>([]);
    const [scripts, setScripts] = useState<string[]>([]);

    // New Job Form
    const [newJob, setNewJob] = useState<Job>({
        id: crypto.randomUUID(),
        script_path: "",
        cron: "0 */5 * * * *",
        status: "active"
    });

    useEffect(() => {
        refresh();
        invoke<string[]>("list_scripts").then(setScripts);
    }, []);

    const refresh = () => {
        invoke<Job[]>("list_jobs").then(setJobs);
    };

    const handleSave = async () => {
        if (!newJob.script_path) return;
        await invoke("save_job", { job: newJob });
        setNewJob({ ...newJob, id: crypto.randomUUID() }); // Reset ID for next
        refresh();
    };

    const handleDelete = async (id: string) => {
        await invoke("delete_job", { jobId: id });
        refresh();
    }

    return (
        <div className="p-4 bg-gray-900 text-white h-full overflow-y-auto">
            <h2 className="text-xl font-bold mb-4">Scheduled Phantoms</h2>

            <div className="bg-gray-800 p-4 rounded mb-6 border border-gray-700">
                <h3 className="text-sm font-bold text-gray-400 mb-2">ADD NEW JOB</h3>
                <div className="grid grid-cols-2 gap-4 mb-2">
                    <div>
                        <label className="text-xs text-gray-500 block">Script</label>
                        <select
                            className="w-full bg-gray-900 border border-gray-600 rounded p-1 text-sm"
                            value={newJob.script_path}
                            onChange={e => setNewJob({...newJob, script_path: e.target.value})}
                        >
                            <option value="">Select Script...</option>
                            {scripts.map(s => <option key={s} value={s}>{s}</option>)}
                        </select>
                    </div>
                    <div>
                        <label className="text-xs text-gray-500 block">Cron Schedule</label>
                        <input
                            className="w-full bg-gray-900 border border-gray-600 rounded p-1 text-sm"
                            value={newJob.cron}
                            onChange={e => setNewJob({...newJob, cron: e.target.value})}
                        />
                    </div>
                </div>
                <button
                    onClick={handleSave}
                    className="bg-blue-600 hover:bg-blue-500 px-3 py-1 rounded text-sm font-bold"
                >
                    Add Job
                </button>
            </div>

            <div className="space-y-2">
                {jobs.map(job => (
                    <div key={job.id} className="bg-gray-800 p-3 rounded border border-gray-700 flex justify-between items-center">
                        <div>
                            <div className="font-bold text-sm text-blue-400">{job.script_path}</div>
                            <div className="text-xs text-gray-500">{job.cron} | {job.status}</div>
                            <div className="text-[10px] text-gray-600">{job.id}</div>
                        </div>
                        <button
                            onClick={() => handleDelete(job.id)}
                            className="text-red-500 hover:text-red-400 text-xs border border-red-900 p-1 rounded"
                        >
                            Delete
                        </button>
                    </div>
                ))}
            </div>
        </div>
    );
}
