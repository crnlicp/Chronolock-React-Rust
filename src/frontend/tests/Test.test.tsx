import { render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import App from '../src/App';
import { StrictMode } from 'react';

describe('App', () => {
  it('renders without crashing', () => {
    render(
      <StrictMode>
        <App />
      </StrictMode>,
    );
    // The app should render without throwing errors
    expect(document.body).toBeTruthy();
  });

  it('renders main app structure', () => {
    const { container } = render(
      <StrictMode>
        <App />
      </StrictMode>,
    );

    // Check that the app container is rendered
    // Since the app uses React Router, we should look for the router structure
    expect(container.firstChild).toBeTruthy();

    // The app should have rendered some content
    expect(container.innerHTML.length).toBeGreaterThan(0);
  });

  it('includes authentication state', () => {
    const { container } = render(
      <StrictMode>
        <App />
      </StrictMode>,
    );

    // In test environment, the app shows authentication loading state
    expect(container.innerHTML).toContain('Authenticating');
  });

  it('renders app structure', () => {
    const { container } = render(
      <StrictMode>
        <App />
      </StrictMode>,
    );

    // The App component includes router and provider structure
    expect(container.firstChild).toBeTruthy();

    // Should have meaningful content in loading state
    expect(container.innerHTML.length).toBeGreaterThan(10);
  });

  it('handles navigation without errors', () => {
    // This test ensures the router setup doesn't cause errors
    const { container } = render(
      <StrictMode>
        <App />
      </StrictMode>,
    );

    // Should render the default route (/) without errors
    expect(container.firstChild).toBeTruthy();
  });
});
