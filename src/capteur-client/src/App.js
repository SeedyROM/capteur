import React from "react";
import { Route, Routes, Outlet } from "react-router-dom";

import NavBar from "./components/ui/NavBar";
import Metrics from "./pages/Metrics";
import Readings from "./pages/Readings";
import Configuration from "./pages/Configuration";

import "./App.css";

function App() {
  function Layout() {
    return (
      <>
        <NavBar />
        <Outlet />
      </>
    );
  }

  return (
    <div className="App">
      <Routes>
        <Route path="/" element={<Layout />}>
          <Route path="/" element={<Readings />} />
          <Route path="/metrics" element={<Metrics />} />
          <Route path="/configuration" element={<Configuration />} />
        </Route>
      </Routes>
    </div>
  );
}

export default App;
