import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { ControlWindow } from "./ControlWindow";
import { WallpaperWindow } from "./WallpaperWindow";
import { isTauriRuntime } from "./taskApi";
import "./App.css";

function App() {
  // İki pencere aynı React bundle'ını yükler. Tauri pencere etiketi hangi
  // kök bileşenin çizileceğini belirler; ayrı frontend projelerine gerek kalmaz.
  const windowLabel = isTauriRuntime() ? getCurrentWebviewWindow().label : "control";

  return windowLabel === "wallpaper" ? <WallpaperWindow /> : <ControlWindow />;
}

export default App;
