import React from "react";

import "./NavBar.css";

const NavBar = () => {
  return (
    <nav class="mx-auto">
      <div class="navbar">
        <div class="navbar__logo">Capteur</div>
        <div class="navbar__items">
          <a href="#!">Readings</a>
          <a href="#!">Metrics</a>
          <a href="#!">Configuration</a>
        </div>
        <div class="navbar__user">
          <a href="#!">Log In</a>
        </div>
      </div>
    </nav>
  );
};

export default NavBar;
