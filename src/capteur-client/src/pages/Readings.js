import React, { useState, useEffect } from "react";
import useWebSocket, { ReadyState } from "react-use-websocket";

import "./Readings.css";

const Readings = () => {
  const [value, setValue] = useState(null);
  const { lastMessage, readyState } = useWebSocket("ws://localhost:9002", {
    retryOnError: true,
    shouldReconnect: (_) => true,
    reconnectInterval: 750,
    reconnectAttempts: Infinity,
  });

  useEffect(() => {
    if (lastMessage === null) return;
    setValue(JSON.parse(lastMessage.data));
  }, [lastMessage]);

  const connectionStatus = {
    [ReadyState.CONNECTING]: "Awaiting connection...",
    [ReadyState.OPEN]: "Connected",
    [ReadyState.CLOSING]: "Closing...",
    [ReadyState.CLOSED]: "Awaiting connection...",
    [ReadyState.UNINSTANTIATED]: "Borked",
  }[readyState];

  return (
    <header className="App-header">
      <div style={{ marginBottom: "2.5rem" }}>
        Status:
        <br />
        <b>{connectionStatus}</b>
      </div>
      <div style={{ opacity: readyState !== ReadyState.OPEN ? 0.5 : 1 }}>
        {value === null ? (
          <div>No sensors to read...</div>
        ) : (
          <table style={{ width: "80vw", tableLayout: "fixed" }}>
            <tr>
              <th>Name</th>
              <th>Value</th>
            </tr>
            {Object.entries(value.reading.sensors).map(([name, reading]) => (
              <tr>
                <td>{name}</td>
                {reading.measurement && (
                  <td>
                    {reading.measurement.value.toFixed(4)}
                    <em>
                      <small>{reading.measurement.unit}</small>
                    </em>
                  </td>
                )}
                {reading.boolean && (
                  <td class={"boolean " + (reading.boolean.value ? "on" : "")}>
                    {reading.boolean.value ? "On" : "Off"}
                  </td>
                )}
              </tr>
            ))}
          </table>
        )}
      </div>
    </header>
  );
};

export default Readings;