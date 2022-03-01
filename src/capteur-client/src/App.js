import React, { useState, useEffect } from "react";
import useWebSocket, { ReadyState } from "react-use-websocket";

import "./App.css";

function App() {
  const [value, setValue] = useState(0);
  const { lastMessage, readyState } = useWebSocket("ws://localhost:9002", {
    retryOnError: true,
    shouldReconnect: (_) => true,
    reconnectInterval: 750,
    reconnectAttempts: Infinity,
  });

  useEffect(() => {
    if (lastMessage === null) return;
    setValue(lastMessage.data);
  }, [lastMessage]);

  const connectionStatus = {
    [ReadyState.CONNECTING]: "Awaiting connection...",
    [ReadyState.OPEN]: "Connected",
    [ReadyState.CLOSING]: "Closing...",
    [ReadyState.CLOSED]: "Awaiting connection...",
    [ReadyState.UNINSTANTIATED]: "Borked",
  }[readyState];

  return (
    <div className="App">
      <header className="App-header">
        <div style={{ marginBottom: "1rem" }}>
          Status:
          <br />
          {connectionStatus}
        </div>
        <div style={{ opacity: readyState !== ReadyState.OPEN ? 0.5 : 1 }}>
          <div>Fake sensor:</div>
          <div>{JSON.stringify(value)}</div>
        </div>
      </header>
    </div>
  );
}

export default App;
