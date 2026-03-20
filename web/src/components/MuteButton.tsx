import { useAudioStore } from "@/lib/audio-store";
import { audio } from "@/lib/audio";

export default function MuteButton() {
  const { muted, toggleMute } = useAudioStore();

  const handleClick = () => {
    toggleMute();
    audio.setMuted(!muted);
  };

  return (
    <button
      onClick={handleClick}
      className="cursor-pointer text-[14px] text-gray-400 transition-colors hover:text-sand"
      title={muted ? "Unmute" : "Mute"}
      aria-label={muted ? "Unmute audio" : "Mute audio"}
    >
      {muted ? "🔇" : "🔊"}
    </button>
  );
}
