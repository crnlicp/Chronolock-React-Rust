import { NavLink } from 'react-router';
import Clock from './Clock';

export const About = () => {
  const targetDate = new Date();
  targetDate.setHours(targetDate.getHours() + 2325);

  return (
    <section id="about">
      {/* About Shortcode */}
      <div className="fn_cs_about">
        <div className="left_part">
          <div className="img bordergradient">
            <div className="img_in" data-bg-img="assets/img/lock.png">
              <Clock targetDate={targetDate} />
              <img src="assets/img/1x1.jpg" alt="" />
            </div>
          </div>
          <div className="bg_overlay">
            <div className="bg_color" />
            <div
              className="bg_image"
              data-bg-img="assets/img/about/about_bg1.png"
            />
          </div>
        </div>
        <div className="right_part">
          <div className="right_in">
            <h3 className="fn__maintitle" data-text="The Rise of Chronolock">
              The Rise of Chronolock
            </h3>
            <div className="fn_cs_divider">
              <div className="divider">
                <span />
                <span />
              </div>
            </div>
            <div className="desc">
              <p>
                Chronolock is built to make digital asset management secure and
                easy for everyone. Using the Internet Computer blockchain, our
                platform lets users create “chronolocks”—time-locked assets that
                are protected by advanced blockchain technology. With a simple
                interface and powerful features, Chronolock helps you control
                when and how your assets are released, giving you peace of mind
                and flexibility.
              </p>
              <p>
                Security is at the heart of Chronolock. We use vetKD encryption,
                a cutting-edge technology on the Internet Computer, to keep your
                chronolocks safe and private. This means your assets are not
                only protected by the blockchain’s transparency and reliability,
                but also by strong encryption that prevents unauthorized access.
                Our team is dedicated to staying ahead in security, so you can
                trust Chronolock with your most important digital assets.
              </p>
              <p>
                At Chronolock, we believe in making blockchain technology
                accessible and useful for everyone. Whether you’re a developer,
                business, or everyday user, our mission is to help you take
                advantage of secure time-locking and encrypted asset management.
                Join us and experience the future of digital security with the
                Internet Computer and vetKD encryption.
              </p>
            </div>
            <NavLink
              to="https://oc.app/"
              className="metaportal_fn_button"
              target="_blank"
              rel="noreferrer"
            >
              <span>Find us On OpenChat</span>
            </NavLink>
          </div>
        </div>
      </div>
      {/* !About Shortcode */}
      <div className="container">
        {/* Mint Shortcode */}
        <div className="fn_cs_mint">
          <div className="left_part">
            <h3
              className="fn__maintitle"
              data-text="How to Create a Chronolock"
            >
              How to Create a Chronolock
            </h3>
            <div className="fn_cs_divider">
              <div className="divider">
                <span />
                <span />
              </div>
            </div>
            <div className="desc">
              <p>
                To get started with creating a chronolock, first log in using
                your Internet Identity. This secure authentication method
                ensures your account and assets are protected on the Internet
                Computer blockchain. Once logged in, navigate to the “Create”
                page from the main menu.
              </p>
              <p>
                On the Create page, you’ll be able to select your recipients and
                set the lock time for your chronolock. This determines who will
                receive the asset and when it will be unlocked. You can also
                choose to include a media file, such as an image or document, to
                personalize your chronolock.
              </p>
              <p>
                After you’ve entered all the details, review your chronolock
                information carefully. When you’re ready, click “Create” to
                finalize and submit your chronolock to the blockchain. Once
                created, you can explore all your chronolocks in the Collection
                page, where you’ll find a complete overview of your time-locked
                assets and their status.
              </p>
              <p>
                After creating your chronolock, you can easily explore and
                manage your time-locked assets in the Collection page. The
                Collection page features tabs such as “Chronolocks” for all
                available assets, “My Chronolocks” for those you have created,
                and “Encrypted for You” for chronolocks where you are the
                recipient. This organized layout helps you quickly find and
                track your assets, ensuring you have access to important details
                and encrypted files whenever you need them.
              </p>
              <p>
                If you have any questions or need assistance, feel free to reach
                out to our support team on OpenChat. We're here to help you make
                the most of your Chronolock experience.
              </p>
            </div>
            <NavLink to="/Create" className="metaportal_fn_button full">
              <span>Create Chronolock</span>
            </NavLink>
          </div>
          <div className="right_part">
            {/* Steps Shortcode */}
            <div className="fn_cs_steps">
              <ul>
                <li>
                  <div className="item">
                    <div className="item_in">
                      <h3 className="fn__gradient_title">01</h3>
                      <p>Login with Internet Identity</p>
                    </div>
                  </div>
                </li>
                <li>
                  <div className="item">
                    <div className="item_in">
                      <h3 className="fn__gradient_title">02</h3>
                      <p>Navigate to Create page</p>
                    </div>
                  </div>
                </li>
                <li>
                  <div className="item">
                    <div className="item_in">
                      <h3 className="fn__gradient_title">03</h3>
                      <p>Select Your Recipients and lock time</p>
                    </div>
                  </div>
                </li>
                <li>
                  <div className="item">
                    <div className="item_in">
                      <h3 className="fn__gradient_title">04</h3>
                      <p>Select any media file to include</p>
                    </div>
                  </div>
                </li>
                <li>
                  <div className="item">
                    <div className="item_in">
                      <h3 className="fn__gradient_title">05</h3>
                      <p>Review your Chronolock and Create</p>
                    </div>
                  </div>
                </li>
                <li>
                  <div className="item">
                    <div className="item_in">
                      <h3 className="fn__gradient_title">06</h3>
                      <p>Explore Chronolocks in Collection page</p>
                    </div>
                  </div>
                </li>
              </ul>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
};
