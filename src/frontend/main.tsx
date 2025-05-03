import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './src/App';
import './src/styles/style.css';
// import './src/styles/index.scss';

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
