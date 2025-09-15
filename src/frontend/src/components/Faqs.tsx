import { useState } from 'react';

export const Faqs = () => {
  const faqs1 = [
    {
      title: 'What is Chronolock?',
      dec: 'Chronolock is a revolutionary platform that allows users to create time-locked digital assets using advanced blockchain technology. By leveraging icrc7 standards and VetKD cryptography, Chronolock ensures that assets can only be accessed or transferred after a specified period, providing enhanced security and control for users.',
    },
    {
      title: 'How does Chronolock ensure security?',
      dec: 'Chronolock employs advanced cryptographic techniques, including icrc7 standards and VetKD cryptography, to secure user assets. By utilizing smart contracts, Chronolock ensures that assets can only be accessed or transferred after a specified period, providing enhanced security and control for users.',
    },
    {
      title: 'What is the login process for Chronolock?',
      dec: 'To log in to Chronolock, users must connect their Internet Identity (II) wallet. This process involves authenticating with the II service, which securely manages user identities and provides a seamless login experience. Once logged in, users can access their time-locked assets and manage their digital portfolio.',
    },
    {
      title: 'How do I create a chronolock?',
      dec: 'To create a chronolock, users must define the asset they want to lock, set the time-lock duration, and specify the conditions for unlocking. This process is facilitated through the Chronolock platform, which provides a user-friendly interface for managing time-locked assets.',
    },
    {
      title: 'What technologies does Chronolock use?',
      dec: 'Chronolock is built on the Internet Computer blockchain, utilizing icrc7 standards for asset management and VetKD cryptography for enhanced security. The platform leverages smart contracts to enforce time-locking mechanisms, ensuring that assets are securely managed and accessible only under specified conditions.',
    },
    {
      title: 'In what condition can I unlock any chronolock?',
      dec: 'A chronolock can be unlocked only after the specified time-lock duration has elapsed and only if you are among the designated recipients. This ensures that the asset remains secure and inaccessible until the predetermined conditions are met.',
    },
  ];
  const faqs2 = [
    {
      title: 'Are there any fees for using Chronolock?',
      dec: 'Media Chronolocks cost 20 $CRNL tokens to create, while standard Chronolocks are free. Additional fees may apply for certain features or services, which will be clearly outlined on the platform.',
    },
    {
      title: 'What do i get if I invite friends to Chronolock?',
      dec: 'For each friend you invite to Chronolock who successfully creates a chronolock, you will receive 20 $CRNL tokens as a reward. This incentive program is designed to encourage users to share the platform with others and help grow the Chronolock community.',
    },
    {
      title: 'Can I modify or cancel a chronolock after it is created?',
      dec: 'Once a chronolock is created, it cannot be modified or canceled. This ensures the integrity and security of the time-locking mechanism, preventing unauthorized access or changes to the locked assets.',
    },
    {
      title: 'How does Chronolock handle privacy?',
      dec: 'Chronolock prioritizes user privacy by implementing robust data protection measures. User identities are managed through the Internet Identity (II) service, ensuring that personal information is not exposed. Additionally, all transactions and interactions with chronolocks are encrypted and securely stored on the blockchain.',
    },
    {
      title: 'What is the benefit of using Chronolock?',
      dec: 'Chronolock offers enhanced security and control over digital assets by implementing time-locking mechanisms. This allows users to manage their assets more effectively, ensuring that they can only be accessed or transferred under specific conditions. Additionally, Chronolock leverages advanced blockchain technology to provide a transparent and tamper-proof environment for asset management.',
    },
    {
      title: 'What are the real world use cases of Chronolock?',
      dec: 'Chronolock can be used in various scenarios, including estate planning, where assets can be time-locked for future beneficiaries; business agreements, where payments or assets are released after certain conditions are met; and personal savings, where individuals can lock away things for future use. Or people can use it for future gifts to their loved ones. The platform’s flexibility and security make it suitable for a wide range of applications.',
    },
  ];

  const [active, setActive] = useState<string | null>();

  const faqMap = (arr: { title: string; dec: string }[], index: string) => {
    return arr.map((data, i) => (
      <div className="fn_cs_accordion" key={i}>
        <div className={`acc_item ${index + i === active ? 'active' : ''}`}>
          <div
            className="acc_header"
            onClick={() =>
              setActive(`${index + i}` === active ? null : `${index + i}`)
            }
          >
            <h3 className="fn__maintitle" data-text={data.title}>
              {data.title}
            </h3>
            <span className="icon">
              <span />
            </span>
          </div>
          <div
            className="acc_content"
            style={{
              display: `${index + i === active ? 'block' : 'none'}`,
            }}
          >
            <p>{data.dec}</p>
          </div>
        </div>
      </div>
    ));
  };
  return (
    <section id="faq">
      <div className="container">
        <div className="fn_cs_faq">
          <div className="faq_col">
            <h3 className="fn__maintitle" data-text="FAQ">
              FAQ
            </h3>
            <div className="fn_cs_divider">
              <div className="divider">
                <span />
                <span />
              </div>
            </div>
            <div className="desc">
              <p>
                Welcome to the Chronolock FAQ section! Here, we address the most
                common questions about our platform, technology, and roadmap.
                Whether you’re a developer, investor, or curious user, this
                section provides clear answers to help you understand our
                mission, features, and how Chronolock leverages cutting-edge
                blockchain and smart contract technology. If you have further
                questions, feel free to reach out to our team or join our
                community channels for more support.
              </p>
            </div>
          </div>
          <div className="faq_col">
            <div className="fn_cs_accordion">{faqMap(faqs1, 'a')}</div>
          </div>
          <div className="faq_col">
            <div className="fn_cs_accordion">{faqMap(faqs2, 'b')}</div>
          </div>
        </div>
      </div>
    </section>
  );
};
