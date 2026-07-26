import React from 'react';
import { Routes, Route, Link, useLocation } from 'react-router-dom';
import ToolList from './pages/ToolList';
import ToolExecute from './pages/ToolExecute';

const API_BASE = import.meta.env.VITE_API_BASE || 'http://127.0.0.1:8080';

export { API_BASE };

function App() {
  const location = useLocation();

  return (
    <div className="app">
      <header className="header">
        <div className="header-content">
          <Link to="/" className="logo">
            <span className="logo-icon">🛠️</span>
            <h1>Trove</h1>
          </Link>
          <nav className="nav">
            <Link
              to="/"
              className={`nav-link ${location.pathname === '/' ? 'active' : ''}`}
            >
              工具列表
            </Link>
          </nav>
        </div>
      </header>
      <main className="main">
        <Routes>
          <Route path="/" element={<ToolList />} />
          <Route path="/tool/:id" element={<ToolExecute />} />
        </Routes>
      </main>
    </div>
  );
}

export default App;
