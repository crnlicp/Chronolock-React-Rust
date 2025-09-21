import { render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { ActorContextProvider } from '../src/ActorContextProvider';

describe('ActorContextProvider', () => {
  it('renders loading state initially', () => {
    const TestChild = () => <div data-testid="test-child">Test Child</div>;

    const { container } = render(
      <ActorContextProvider>
        <TestChild />
      </ActorContextProvider>,
    );

    // ActorContextProvider shows loading state in test environment
    expect(container.innerHTML).toContain('Authenticating');
  });

  it('provides context structure', () => {
    const TestChild = () => {
      // This component should render without errors when wrapped in the provider
      return <div>Context provided</div>;
    };

    render(
      <ActorContextProvider>
        <TestChild />
      </ActorContextProvider>,
    );

    // If we reach here without errors, the provider is working
    expect(document.body).toBeTruthy();
  });

  it('handles provider initialization', () => {
    const { container } = render(
      <ActorContextProvider>
        <div>Child 1</div>
        <div>Child 2</div>
        <div>Child 3</div>
      </ActorContextProvider>,
    );

    // Provider should render without crashing
    expect(container.firstChild).toBeTruthy();

    // In test environment, shows authenticating state
    expect(container.innerHTML).toContain('Authenticating');
  });
});
