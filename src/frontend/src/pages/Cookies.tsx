import { useEffect } from 'react';

export const Cookies = () => {
  useEffect(() => {
    window.scrollTo(0, 0);
  }, []);

  return (
    <div className="container page_container">
      <div style={{ padding: '40px 20px', maxWidth: '800px', margin: '0 auto' }}>
        <h1 style={{ textAlign: 'center', marginBottom: '40px', color: '#fff' }}>
          Cookie Policy
        </h1>
        
        <div style={{ color: '#ccc', lineHeight: '1.6' }}>
          <p style={{ marginBottom: '20px' }}>
            <strong>Last updated:</strong> {new Date().toLocaleDateString()}
          </p>

          <section style={{ marginBottom: '30px' }}>
            <h2 style={{ color: '#fff', marginBottom: '15px' }}>1. What Are Cookies?</h2>
            <p>
              Cookies are small text files that are placed on your device when you visit our website. 
              They help us provide you with a better experience by remembering your preferences 
              and understanding how you use our Chronolock platform.
            </p>
          </section>

          <section style={{ marginBottom: '30px' }}>
            <h2 style={{ color: '#fff', marginBottom: '15px' }}>2. How We Use Cookies</h2>
            <p>
              We use cookies for several purposes:
            </p>
            <ul style={{ marginLeft: '20px' }}>
              <li><strong>Essential Cookies:</strong> Required for the website to function properly</li>
              <li><strong>Functionality Cookies:</strong> Remember your preferences and settings</li>
              <li><strong>Analytics Cookies:</strong> Help us understand how visitors use our site</li>
              <li><strong>Performance Cookies:</strong> Improve website speed and performance</li>
            </ul>
          </section>

          <section style={{ marginBottom: '30px' }}>
            <h2 style={{ color: '#fff', marginBottom: '15px' }}>3. Types of Cookies We Use</h2>
            
            <h3 style={{ color: '#fff', marginBottom: '10px' }}>3.1 Strictly Necessary Cookies</h3>
            <p>
              These cookies are essential for the website to function and cannot be switched off. 
              They include:
            </p>
            <ul style={{ marginLeft: '20px', marginBottom: '15px' }}>
              <li>Session management cookies</li>
              <li>Security cookies</li>
              <li>Load balancing cookies</li>
              <li>Wallet connection state cookies</li>
            </ul>

            <h3 style={{ color: '#fff', marginBottom: '10px' }}>3.2 Functional Cookies</h3>
            <p>
              These cookies enable enhanced functionality and personalization:
            </p>
            <ul style={{ marginLeft: '20px', marginBottom: '15px' }}>
              <li>Language preferences</li>
              <li>Theme settings</li>
              <li>User interface preferences</li>
              <li>Previously selected options</li>
            </ul>

            <h3 style={{ color: '#fff', marginBottom: '10px' }}>3.3 Analytics Cookies</h3>
            <p>
              These cookies help us understand how visitors interact with our website:
            </p>
            <ul style={{ marginLeft: '20px', marginBottom: '15px' }}>
              <li>Google Analytics (if implemented)</li>
              <li>User behavior tracking</li>
              <li>Performance metrics</li>
              <li>Error reporting</li>
            </ul>
          </section>

          <section style={{ marginBottom: '30px' }}>
            <h2 style={{ color: '#fff', marginBottom: '15px' }}>4. Blockchain and Decentralized Storage</h2>
            <p>
              As a decentralized application built on the Internet Computer Protocol (ICP), 
              most of your data is stored on the blockchain rather than in traditional cookies. 
              However, we may still use cookies for:
            </p>
            <ul style={{ marginLeft: '20px' }}>
              <li>Caching blockchain data for improved performance</li>
              <li>Storing temporary encryption keys</li>
              <li>Managing wallet connection status</li>
              <li>Optimizing user experience</li>
            </ul>
          </section>

          <section style={{ marginBottom: '30px' }}>
            <h2 style={{ color: '#fff', marginBottom: '15px' }}>5. Third-Party Cookies</h2>
            <p>
              We may use third-party services that set their own cookies:
            </p>
            <ul style={{ marginLeft: '20px' }}>
              <li><strong>Wallet Providers:</strong> Internet Identity, Plug Wallet, etc.</li>
              <li><strong>Analytics Services:</strong> For understanding user behavior</li>
              <li><strong>CDN Providers:</strong> For content delivery optimization</li>
              <li><strong>Social Media:</strong> For sharing functionality (if implemented)</li>
            </ul>
            <p>
              These third parties have their own cookie policies, which we encourage you to review.
            </p>
          </section>

          <section style={{ marginBottom: '30px' }}>
            <h2 style={{ color: '#fff', marginBottom: '15px' }}>6. Managing Your Cookie Preferences</h2>
            <p>
              You can control and manage cookies in several ways:
            </p>
            
            <h3 style={{ color: '#fff', marginBottom: '10px' }}>6.1 Browser Settings</h3>
            <p>
              Most browsers allow you to:
            </p>
            <ul style={{ marginLeft: '20px', marginBottom: '15px' }}>
              <li>View and delete cookies</li>
              <li>Block cookies from specific sites</li>
              <li>Block third-party cookies</li>
              <li>Clear cookies when you close your browser</li>
            </ul>

            <h3 style={{ color: '#fff', marginBottom: '10px' }}>6.2 Our Platform</h3>
            <p>
              We may provide cookie preference controls within our application to help you 
              manage non-essential cookies while maintaining functionality.
            </p>
          </section>

          <section style={{ marginBottom: '30px' }}>
            <h2 style={{ color: '#fff', marginBottom: '15px' }}>7. Cookie Retention</h2>
            <p>
              Different cookies have different lifespans:
            </p>
            <ul style={{ marginLeft: '20px' }}>
              <li><strong>Session Cookies:</strong> Deleted when you close your browser</li>
              <li><strong>Persistent Cookies:</strong> Remain until they expire or you delete them</li>
              <li><strong>Functional Cookies:</strong> Typically retained for 30-365 days</li>
              <li><strong>Analytics Cookies:</strong> Usually retained for 1-2 years</li>
            </ul>
          </section>

          <section style={{ marginBottom: '30px' }}>
            <h2 style={{ color: '#fff', marginBottom: '15px' }}>8. Updates to This Policy</h2>
            <p>
              We may update this Cookie Policy from time to time to reflect changes in our practices 
              or for other operational, legal, or regulatory reasons. We will post the updated policy 
              on this page and update the "Last updated" date.
            </p>
          </section>

          <section style={{ marginBottom: '30px' }}>
            <h2 style={{ color: '#fff', marginBottom: '15px' }}>9. Contact Us</h2>
            <p>
              If you have any questions about our use of cookies or this Cookie Policy, 
              please contact us through our official channels or community forums.
            </p>
          </section>
        </div>
      </div>
    </div>
  );
};