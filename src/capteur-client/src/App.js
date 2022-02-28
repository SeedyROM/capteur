import React, { useState, useEffect } from "react";
import useWebSocket from "react-use-websocket";

import "./App.css";

function App() {
  const [value, setValue] = useState(0);
  const { lastMessage } = useWebSocket("ws://localhost:9002");

  useEffect(() => {
    if (lastMessage === null) return;
    setValue(lastMessage.data);
  }, [lastMessage]);

  return (
    <div className="App">
      <header className="App-header">
        <div>Fake sensor:</div>
        <div>{value}</div>
      </header>
    </div>
  );
}

export default App;
