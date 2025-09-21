import { render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { Home } from '../src/pages/Home';
import { ActorContextProvider } from '../src/ActorContextProvider';
import { MemoryRouter } from 'react-router';

describe('Home Page', () => {
  const renderWithProviders = (component: React.ReactElement) => {
    return render(
      <ActorContextProvider>
        <MemoryRouter>{component}</MemoryRouter>
      </ActorContextProvider>,
    );
  };

  it('renders without crashing', () => {
    renderWithProviders(<Home />);
    expect(document.body).toBeTruthy();
  });

  it('renders with provider context', () => {
    const { container } = renderWithProviders(<Home />);

    // In test environment, shows loading/authenticating state
    expect(container.innerHTML).toContain('Authenticating');
  });

  it('handles component initialization', () => {
    const { container } = renderWithProviders(<Home />);

    // Component should render without errors
    expect(container.firstChild).toBeTruthy();

    // Should have some rendered content
    expect(container.innerHTML.length).toBeGreaterThan(0);
  });
});
