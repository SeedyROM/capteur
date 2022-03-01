import React from "react";
import { ReadyState } from "react-use-websocket";
import { useWSPassthrough } from "../contexts/WSPassthrough";

import "./Readings.css";

const Readings = () => {
  const { value, connectionStatus, readyState } = useWSPassthrough();

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
