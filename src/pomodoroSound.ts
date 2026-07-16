let audioContext: AudioContext | null = null;

export async function playPomodoroChime(volume: number) {
  audioContext ??= new window.AudioContext();
  if (audioContext.state === "suspended") await audioContext.resume();

  const start = audioContext.currentTime;
  const master = audioContext.createGain();
  master.gain.setValueAtTime(0.0001, start);
  master.gain.exponentialRampToValueAtTime(Math.max(0.0001, volume / 100 * 0.16), start + 0.025);
  master.gain.exponentialRampToValueAtTime(0.0001, start + 0.62);
  master.connect(audioContext.destination);

  [659.25, 783.99, 987.77].forEach((frequency, index) => {
    const oscillator = audioContext!.createOscillator();
    const gain = audioContext!.createGain();
    const noteStart = start + index * 0.11;
    oscillator.type = "sine";
    oscillator.frequency.setValueAtTime(frequency, noteStart);
    gain.gain.setValueAtTime(0.0001, noteStart);
    gain.gain.exponentialRampToValueAtTime(0.85, noteStart + 0.018);
    gain.gain.exponentialRampToValueAtTime(0.0001, noteStart + 0.28);
    oscillator.connect(gain);
    gain.connect(master);
    oscillator.start(noteStart);
    oscillator.stop(noteStart + 0.3);
  });
}
