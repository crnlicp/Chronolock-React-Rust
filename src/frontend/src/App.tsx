import { ChangeEvent, useState } from 'react';
import './styles/App.scss';
import rustLogo from '../assets/rust.svg';
import reactLogo from '../assets/react.svg';
import ethLogo from '../assets/eth.svg';
import { hello_world } from '../../declarations/hello_world';

function App() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | undefined>();
  const [name, setName] = useState<string>('');
  const [response, setResponse] = useState<string>('');

  const fetchResponse = async () => {
    try {
      setLoading(true);
      setError(undefined);
      const res = await hello_world.greet(name);
      setResponse(res);
    } catch (err) {
      console.error(err);
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  const handleChangeText = (
    event: ChangeEvent<HTMLInputElement> | undefined,
  ): void => {
    if (!event?.target.value) {
      return;
    }
    setName(event.target.value);
  };

  return (
    <div className="App">
      <div>
        <a href="https://reactjs.org" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
        <a
          href="https://github.com/internet-computer-protocol/evm-rpc-canister#readme"
          target="_blank"
        >
          <img src={ethLogo} className="logo ethereum" alt="Ethereum logo" />
        </a>
        <a
          href="https://internetcomputer.org/docs/current/developer-docs/backend/rust/"
          target="_blank"
        >
          <span className="logo-stack">
            <img src={rustLogo} className="logo rust" alt="Rust logo" />
          </span>
        </a>
      </div>
      <h1 style={{ paddingLeft: 36 }}>React + EVM RPC + Rust</h1>
      <input type="text" onChange={handleChangeText} value={name} />
      <div className="card" style={{ opacity: loading ? 0.5 : 1 }}>
        <button onClick={fetchResponse}>Get Backend Response</button>

        {!!error && (
          <pre style={{ textAlign: 'left', color: 'grey' }}>{error}</pre>
        )}
        {!!loading && !error && <div className="loader" />}
        {!!response && <div>{response}</div>}
      </div>
    </div>
  );
}

export default App;
