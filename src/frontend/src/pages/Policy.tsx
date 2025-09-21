import { useEffect } from 'react';

export const Policy = () => {
  useEffect(() => {
    window.scrollTo(0, 0);
  }, []);

  return (
    <div className="container page_container">
      <div style={{ padding: '40px 20px', maxWidth: '800px', margin: '0 auto' }}>
        <h1 style={{ textAlign: 'center', marginBottom: '40px', color: '#fff' }}>
          Privacy Policy
        </h1>
        
        <div style={{ color: '#ccc', lineHeight: '1.6' }}>
          <p style={{ marginBottom: '20px' }}>
            <strong>Last updated:</strong> {new Date().toLocaleDateString()}
          </p>

          <section style={{ marginBottom: '30px' }}>
            <h2 style={{ color: '#fff', marginBottom: '15px' }}>1. Introduction</h2>
            <p>
              Welcome to Chronolock. We are committed to protecting your privacy and personal data. 
              This Privacy Policy explains how we collect, use, and protect your information when you use our 
              decentralized time-locked cryptographic platform built on the Internet Computer Protocol (ICP).
            </p>
          </section>

          <section style={{ marginBottom: '30px' }}>
            <h2 style={{ color: '#fff', marginBottom: '15px' }}>2. Information We Collect</h2>
            <h3 style={{ color: '#fff', marginBottom: '10px' }}>2.1 Blockchain Data</h3>
            <p>
              As a blockchain-based application, certain information is stored on the Internet Computer blockchain:
            </p>
            <ul style={{ marginLeft: '20px', marginBottom: '15px' }}>
              <li>Wallet addresses and transaction data</li>
              <li>Encrypted content and metadata</li>
              <li>Time-lock configurations</li>
              <li>Smart contract interactions</li>
            </ul>
            
            <h3 style={{ color: '#fff', marginBottom: '10px' }}>2.2 Technical Data</h3>
            <p>
              We may collect technical information to improve our service:
            </p>
            <ul style={{ marginLeft: '20px' }}>
              <li>Browser type and version</li>
              <li>Device information</li>
              <li>Usage patterns and analytics</li>
              <li>Performance metrics</li>
            </ul>
          </section>

          <section style={{ marginBottom: '30px' }}>
            <h2 style={{ color: '#fff', marginBottom: '15px' }}>3. How We Use Your Information</h2>
            <ul style={{ marginLeft: '20px' }}>
              <li>To provide and maintain our time-locking services</li>
              <li>To execute smart contracts and time-lock mechanisms</li>
              <li>To improve platform security and functionality</li>
              <li>To provide customer support</li>
              <li>To comply with legal obligations</li>
            </ul>
          </section>

          <section style={{ marginBottom: '30px' }}>
            <h2 style={{ color: '#fff', marginBottom: '15px' }}>4. Data Security</h2>
            <p>
              We implement robust security measures to protect your data:
            </p>
            <ul style={{ marginLeft: '20px' }}>
              <li>End-to-end encryption for all content</li>
              <li>Decentralized storage on ICP blockchain</li>
              <li>Advanced cryptographic key management</li>
              <li>Regular security audits and updates</li>
            </ul>
          </section>

          <section style={{ marginBottom: '30px' }}>
            <h2 style={{ color: '#fff', marginBottom: '15px' }}>5. Your Rights</h2>
            <p>
              You have the following rights regarding your personal data:
            </p>
            <ul style={{ marginLeft: '20px' }}>
              <li>Access to your personal data</li>
              <li>Correction of inaccurate data</li>
              <li>Data portability</li>
              <li>Withdrawal of consent (where applicable)</li>
            </ul>
            <p>
              Please note that due to the immutable nature of blockchain technology, 
              some data cannot be modified or deleted once recorded.
            </p>
          </section>

          <section style={{ marginBottom: '30px' }}>
            <h2 style={{ color: '#fff', marginBottom: '15px' }}>6. Third-Party Services</h2>
            <p>
              Our platform may integrate with third-party services such as:
            </p>
            <ul style={{ marginLeft: '20px' }}>
              <li>Wallet providers</li>
              <li>Analytics services</li>
              <li>Infrastructure providers</li>
            </ul>
            <p>
              These services have their own privacy policies, which we encourage you to review.
            </p>
          </section>

          <section style={{ marginBottom: '30px' }}>
            <h2 style={{ color: '#fff', marginBottom: '15px' }}>7. International Transfers</h2>
            <p>
              As a decentralized application running on the Internet Computer, 
              your data may be processed across multiple jurisdictions. 
              We ensure appropriate safeguards are in place for international data transfers.
            </p>
          </section>

          <section style={{ marginBottom: '30px' }}>
            <h2 style={{ color: '#fff', marginBottom: '15px' }}>8. Changes to This Policy</h2>
            <p>
              We may update this Privacy Policy from time to time. 
              We will notify you of any material changes by posting the new policy on this page 
              and updating the "Last updated" date.
            </p>
          </section>

          <section style={{ marginBottom: '30px' }}>
            <h2 style={{ color: '#fff', marginBottom: '15px' }}>9. Contact Us</h2>
            <p>
              If you have any questions about this Privacy Policy or our data practices, 
              please contact us through our official channels or community forums.
            </p>
          </section>
        </div>
      </div>
    </div>
  );
};