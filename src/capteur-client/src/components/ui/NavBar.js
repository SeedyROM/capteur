import React from "react";
import { NavLink } from "react-router-dom";

import "./NavBar.css";

const navLinks = [
  {
    path: "/",
    name: "Readings",
  },
  {
    path: "/metrics",
    name: "Metrics",
  },
  {
    path: "/configuration",
    name: "Configuration",
  },
];

const NavBar = () => {
  return (
    <nav className="mx-auto">
      <div className="navbar">
        <div className="navbar__logo">Capteur</div>
        <div className="navbar__items">
          {navLinks.map((link) => (
            <NavLink
              to={link.path}
              className={({ isActive }) => (isActive ? "active" : "")}
            >
              {link.name}
            </NavLink>
          ))}
        </div>
        <div className="navbar__user">
          <a href="#!">Log In</a>
        </div>
      </div>
    </nav>
  );
};

export default NavBar;
