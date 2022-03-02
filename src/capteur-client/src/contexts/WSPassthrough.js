import { useState, useEffect, createContext, useMemo, useContext } from "react";
import useWebSocket, { ReadyState } from "react-use-websocket";

export const WSPassthroughContext = createContext();

const WSPassthrough = (props) => {
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
    [ReadyState.UNINSTANTIATED]: "Borked!",
  }[readyState];

  const contextValue = useMemo(
    () => ({
      value,
      readyState,
      connectionStatus,
    }),
    [value, connectionStatus, readyState]
  );

  return (
    <WSPassthroughContext.Provider value={contextValue}>
      {props.children}
    </WSPassthroughContext.Provider>
  );
};

export const useWSPassthrough = () => {
  return useContext(WSPassthroughContext);
};

export default WSPassthrough;
