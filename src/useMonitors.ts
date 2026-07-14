import { useEffect, useState } from "react";
import { listMonitors } from "./taskApi";
import type { MonitorInfo } from "./types";

export function useMonitors() {
  const [monitors, setMonitors] = useState<MonitorInfo[]>([]);
  const [monitorError, setMonitorError] = useState("");

  useEffect(() => {
    listMonitors()
      .then((items) => {
        setMonitors(items);
        setMonitorError("");
      })
      .catch((reason) => setMonitorError(String(reason)));
  }, []);

  return { monitors, monitorError };
}
