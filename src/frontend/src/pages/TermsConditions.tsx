import { useEffect } from 'react';

export const TermsConditions = () => {
  useEffect(() => {
    window.scrollTo(0, 0);
  }, []);

  return (
    <div className="container page_container">
      <div style={{ padding: '40px 20px', maxWidth: '800px', margin: '0 auto' }}>
        <h1 style={{ textAlign: 'center', marginBottom: '40px', color: '#fff' }}>
          Terms & Conditions
        </h1>
        
        <div style={{ color: '#ccc', lineHeight: '1.6' }}>
          <p style={{ marginBottom: '20px' }}>
            <strong>Last updated:</strong> {new Date().toLocaleDateString()}
          </p>

          <section style={{ marginBottom: '30px' }}>
            <h2 style={{ color: '#fff', marginBottom: '15px' }}>1. Agreement to Terms</h2>
            <p>
              By accessing and using Chronolock, you accept and agree to be bound by the terms 
              and provision of this agreement. If you do not agree to abide by the above, 
              please do not use this service.
            </p>
          </section>

          <section style={{ marginBottom: '30px' }}>
            <h2 style={{ color: '#fff', marginBottom: '15px' }}>2. About Chronolock</h2>
            <p>
              Chronolock is a decentralized time-locked cryptographic platform built on the 
              Internet Computer Protocol (ICP). Our platform allows users to encrypt and 
              time-lock digital content, ensuring it can only be accessed after specified 
              time conditions are met.
            </p>
          </section>

          <section style={{ marginBottom: '30px' }}>
            <h2 style={{ color: '#fff', marginBottom: '15px' }}>3. Service Description</h2>
            <p>
              Chronolock provides the following services:
            </p>
            <ul style={{ marginLeft: '20px' }}>
              <li>Time-locked encryption and decryption of digital content</li>
              <li>Secure storage on the Internet Computer blockchain</li>
              <li>Advanced cryptographic key management</li>
              <li>Decentralized access control mechanisms</li>
              <li>NFT-based representation of time-locked content</li>
            </ul>
          </section>

          <section style={{ marginBottom: '30px' }}>
            <h2 style={{ color: '#fff', marginBottom: '15px' }}>4. User Responsibilities</h2>
            <p>
              As a user of Chronolock, you agree to:
            </p>
            <ul style={{ marginLeft: '20px' }}>
              <li>Provide accurate and truthful information</li>
              <li>Maintain the security of your wallet and private keys</li>
              <li>Use the service in compliance with all applicable laws</li>
              <li>Not attempt to circumvent time-lock mechanisms</li>
              <li>Not upload illegal, harmful, or infringing content</li>
              <li>Respect the intellectual property rights of others</li>
              <li>Not interfere with the operation of the platform</li>
            </ul>
          </section>

          <section style={{ marginBottom: '30px' }}>
            <h2 style={{ color: '#fff', marginBottom: '15px' }}>5. Prohibited Content</h2>
            <p>
              You may not use Chronolock to store or transmit:
            </p>
            <ul style={{ marginLeft: '20px' }}>
              <li>Illegal or unlawful content</li>
              <li>Content that infringes on intellectual property rights</li>
              <li>Malicious software or code</li>
              <li>Content that violates privacy rights</li>
              <li>Spam or unsolicited communications</li>
              <li>Content that promotes violence or hatred</li>
              <li>Personally identifiable information of others without consent</li>
            </ul>
          </section>

          <section style={{ marginBottom: '30px' }}>
            <h2 style={{ color: '#fff', marginBottom: '15px' }}>6. Blockchain and Decentralization</h2>
            <p>
              You understand and acknowledge that:
            </p>
            <ul style={{ marginLeft: '20px' }}>
              <li>Chronolock operates on the Internet Computer blockchain</li>
              <li>Blockchain transactions are irreversible</li>
              <li>Content stored on the blockchain is persistent and immutable</li>
              <li>We cannot reverse or modify blockchain transactions</li>
              <li>Network fees may apply for blockchain operations</li>
              <li>The platform may be affected by blockchain network conditions</li>
            </ul>
          </section>

          <section style={{ marginBottom: '30px' }}>
            <h2 style={{ color: '#fff', marginBottom: '15px' }}>7. Security and Risk Acknowledgment</h2>
            <p>
              You acknowledge and accept the following risks:
            </p>
            <ul style={{ marginLeft: '20px' }}>
              <li>Cryptographic systems may have vulnerabilities</li>
              <li>Private keys must be kept secure and cannot be recovered if lost</li>
              <li>Time-lock mechanisms depend on accurate time sources</li>
              <li>Blockchain networks may experience downtime or congestion</li>
              <li>Software may contain bugs or errors</li>
              <li>Regulatory changes may affect service availability</li>
            </ul>
          </section>

          <section style={{ marginBottom: '30px' }}>
            <h2 style={{ color: '#fff', marginBottom: '15px' }}>8. Intellectual Property</h2>
            <p>
              The Chronolock platform, including its code, design, and documentation, 
              is protected by intellectual property laws. You retain ownership of content 
              you upload, but grant us the necessary rights to provide our services.
            </p>
          </section>

          <section style={{ marginBottom: '30px' }}>
            <h2 style={{ color: '#fff', marginBottom: '15px' }}>9. Disclaimer of Warranties</h2>
            <p>
              Chronolock is provided "as is" without any warranties, express or implied. 
              We do not warrant that:
            </p>
            <ul style={{ marginLeft: '20px' }}>
              <li>The service will be uninterrupted or error-free</li>
              <li>Defects will be corrected</li>
              <li>The service is free of viruses or harmful components</li>
              <li>Time-lock mechanisms will function perfectly in all circumstances</li>
              <li>Blockchain networks will remain stable and accessible</li>
            </ul>
          </section>

          <section style={{ marginBottom: '30px' }}>
            <h2 style={{ color: '#fff', marginBottom: '15px' }}>10. Limitation of Liability</h2>
            <p>
              To the maximum extent permitted by law, Chronolock and its developers 
              shall not be liable for any direct, indirect, incidental, special, 
              consequential, or punitive damages, including but not limited to:
            </p>
            <ul style={{ marginLeft: '20px' }}>
              <li>Loss of profits or data</li>
              <li>Business interruption</li>
              <li>Lost private keys or access to content</li>
              <li>Blockchain network failures</li>
              <li>Cryptographic vulnerabilities</li>
            </ul>
          </section>

          <section style={{ marginBottom: '30px' }}>
            <h2 style={{ color: '#fff', marginBottom: '15px' }}>11. Indemnification</h2>
            <p>
              You agree to indemnify and hold harmless Chronolock, its developers, 
              and affiliates from any claims, damages, or expenses arising from:
            </p>
            <ul style={{ marginLeft: '20px' }}>
              <li>Your use of the platform</li>
              <li>Your violation of these terms</li>
              <li>Your infringement of any rights of another</li>
              <li>Content you upload or transmit</li>
            </ul>
          </section>

          <section style={{ marginBottom: '30px' }}>
            <h2 style={{ color: '#fff', marginBottom: '15px' }}>12. Privacy and Data Protection</h2>
            <p>
              Your privacy is important to us. Please review our Privacy Policy, 
              which also governs your use of the service, to understand our practices.
            </p>
          </section>

          <section style={{ marginBottom: '30px' }}>
            <h2 style={{ color: '#fff', marginBottom: '15px' }}>13. Modifications to Terms</h2>
            <p>
              We reserve the right to modify these terms at any time. 
              Changes will be effective immediately upon posting. 
              Your continued use of the service constitutes acceptance of the modified terms.
            </p>
          </section>

          <section style={{ marginBottom: '30px' }}>
            <h2 style={{ color: '#fff', marginBottom: '15px' }}>14. Termination</h2>
            <p>
              We may terminate or suspend your access to the service immediately, 
              without prior notice, if you breach these terms. 
              Content stored on the blockchain will remain according to blockchain rules.
            </p>
          </section>

          <section style={{ marginBottom: '30px' }}>
            <h2 style={{ color: '#fff', marginBottom: '15px' }}>15. Governing Law</h2>
            <p>
              These terms shall be governed by and construed in accordance with applicable laws. 
              Any disputes arising from these terms or your use of the service shall be subject 
              to the jurisdiction of appropriate courts.
            </p>
          </section>

          <section style={{ marginBottom: '30px' }}>
            <h2 style={{ color: '#fff', marginBottom: '15px' }}>16. Contact Information</h2>
            <p>
              If you have any questions about these Terms & Conditions, 
              please contact us through our official channels or community forums.
            </p>
          </section>

          <section style={{ marginBottom: '30px' }}>
            <h2 style={{ color: '#fff', marginBottom: '15px' }}>17. Severability</h2>
            <p>
              If any provision of these terms is found to be unenforceable, 
              the remaining provisions will remain in full force and effect.
            </p>
          </section>
        </div>
      </div>
    </div>
  );
};