import '@testing-library/jest-dom/vitest';

if (typeof document !== 'undefined') {
  // Simulate the ic_env cookie the frontend canister sets (canister id + root key).
  document.cookie = `ic_env=${encodeURIComponent(
    'PUBLIC_CANISTER_ID:backend=uxrrr-q7777-77774-qaaaq-cai&ic_root_key=308182301d060d2b0601040182dc7c0503010201060c2b0601040182dc7c05030201036100814c0e6ec71fab583b08bd81373c255c3c371b2e84863c98a4f1e08b74235d14fb5d9c0cd546d9685f913a0c0b2cc5341583bf4b4392e467db96d65b9bb4cb717112f8472e0d5a4d14505ffd7484b01291091c5f87b98883463f98091a0baaae',
  )}`;
}
