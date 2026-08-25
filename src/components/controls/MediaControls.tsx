import { useState } from 'react';
import { Play, Pause, SkipBack, SkipForward, Volume2, Volume1, Moon, Sun } from 'lucide-react';
import { tauriApi } from '../../lib/ipc';

interface ControlsProps {
  serial: string;
}

export const MediaControls: React.FC<ControlsProps> = ({ serial }) => {
  const [isPlaying, setIsPlaying] = useState(false);

  const handleAction = async (name: string, fn: () => Promise<void>) => {
    try {
      await fn();
    } catch (e) {
      console.error(`Action ${name} failed:`, e);
    }
  };

  return (
    <div className="bg-[#181a20] border border-[#262933] rounded-xl p-4 flex flex-col gap-3">
      <div className="flex items-center justify-between">
        <span className="text-xs font-semibold uppercase tracking-wider text-gray-400">Quick Controls</span>
        <span className="text-[11px] text-gray-500 font-mono">ADB Input</span>
      </div>

      <div className="grid grid-cols-2 gap-3">
        {/* Media Group */}
        <div className="flex items-center justify-between bg-[#111317] border border-[#20222a] rounded-lg p-1.5">
          <button
            onClick={() => handleAction('prev', () => tauriApi.mediaPrev(serial))}
            className="p-2 hover:bg-[#262933] rounded-md text-gray-300 hover:text-white transition active:scale-95"
            title="Previous Track"
          >
            <SkipBack size={16} />
          </button>
          <button
            onClick={() =>
              handleAction('play', async () => {
                await tauriApi.mediaPlayPause(serial);
                setIsPlaying(!isPlaying);
              })
            }
            className="p-2 bg-indigo-600 hover:bg-indigo-500 rounded-md text-white transition active:scale-95 shadow-sm"
            title="Play / Pause"
          >
            {isPlaying ? <Pause size={16} /> : <Play size={16} />}
          </button>
          <button
            onClick={() => handleAction('next', () => tauriApi.mediaNext(serial))}
            className="p-2 hover:bg-[#262933] rounded-md text-gray-300 hover:text-white transition active:scale-95"
            title="Next Track"
          >
            <SkipForward size={16} />
          </button>
        </div>

        {/* Volume Group */}
        <div className="flex items-center justify-between bg-[#111317] border border-[#20222a] rounded-lg p-1.5">
          <button
            onClick={() => handleAction('voldown', () => tauriApi.volumeDown(serial))}
            className="p-2 hover:bg-[#262933] rounded-md text-gray-300 hover:text-white transition active:scale-95"
            title="Volume Down"
          >
            <Volume1 size={16} />
          </button>
          <button
            onClick={() => handleAction('volup', () => tauriApi.volumeUp(serial))}
            className="p-2 hover:bg-[#262933] rounded-md text-gray-300 hover:text-white transition active:scale-95"
            title="Volume Up"
          >
            <Volume2 size={16} />
          </button>
          <button
            onClick={() => handleAction('wake', () => tauriApi.wakeScreen(serial))}
            className="p-2 hover:bg-amber-500/20 text-amber-400 rounded-md transition active:scale-95"
            title="Wake Screen"
          >
            <Sun size={16} />
          </button>
          <button
            onClick={() => handleAction('lock', () => tauriApi.lockScreen(serial))}
            className="p-2 hover:bg-red-500/20 text-red-400 rounded-md transition active:scale-95"
            title="Lock Display"
          >
            <Moon size={16} />
          </button>
        </div>
      </div>
    </div>
  );
};
