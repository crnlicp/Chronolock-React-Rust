import CustomizedSteppers from '../components/create/Stepper';
import { useState } from 'react';
import { UnlockTimeAndRecipients } from '../components/create/UnlockTimeAndRecipients';

export const Create = () => {
  const [activeStep, setActiveStep] = useState(0);
  // const targetDate = new Date();
  // targetDate.setHours(targetDate.getHours() + 2325);

  const handleNext = () => {
    setActiveStep((prevActiveStep) => prevActiveStep + 1);
  };
  const handleBack = () => {
    setActiveStep((prevActiveStep) => prevActiveStep - 1);
  };

  const handleDateChange = (date: string | null): void => {
    if (date) {
      const parsedDate = new Date(date);
      if (!isNaN(parsedDate.getTime())) {
        console.log('Selected date:', parsedDate);
      } else {
        console.error('Invalid date format:', date);
      }
    } else {
      console.warn('No date selected');
    }
  };

  const handleRecipientChange = (recipients: string[]): void => {
    console.log('Selected recipients:', recipients);
  };

  return (
    <div className="container page_container">
      <CustomizedSteppers activeStep={activeStep} />
      {activeStep === 0 && (
        <UnlockTimeAndRecipients
          onNext={handleNext}
          onDateChange={handleDateChange}
          onRecipientsChange={handleRecipientChange}
        />
      )}
      {activeStep === 1 && (
        <div className="container small">
          <h3>Upload Your File</h3>
          <p>Upload your file here...</p>
          <button onClick={handleNext}>Next</button>
          <button onClick={handleBack}>Back</button>
        </div>
      )}
      {activeStep === 2 && (
        <div className="container small">
          <h3>Fill in the details</h3>
          <p>Fill in the details here...</p>
          <button onClick={handleNext}>Finish</button>
          <button onClick={handleBack}>Back</button>
        </div>
      )}
      {/* Uncomment the following code to display the minting section */}
      {/* <div className="container small">
        <div className="metaportal_fn_mint_top">
          <div className="mint_left">
            <div className="img">
              <div className="img_in" data-bg-img="assets/img/about/1.jpg">
                <img src="/img/1x1.jpg" alt="" />
              </div>
            </div>
          </div>
          <div className="mint_right">
            <div className="metaportal_fn_share">
              <h5 className="label">Share:</h5>
              <ul>
                <li>
                  <a href="#">
                    <img
                      src="assets/svg/social/twitter-1.svg"
                      alt=""
                      className="fn__svg"
                    />
                  </a>
                </li>
                <li>
                  <a href="#">
                    <img
                      src="assets/svg/social/facebook-1.svg"
                      alt=""
                      className="fn__svg"
                    />
                  </a>
                </li>
                <li>
                  <a href="#">
                    <img
                      src="assets/svg/social/instagram-1.svg"
                      alt=""
                      className="fn__svg"
                    />
                  </a>
                </li>
                <li>
                  <a href="#">
                    <img
                      src="assets/svg/social/pinterest-1.svg"
                      alt=""
                      className="fn__svg"
                    />
                  </a>
                </li>
                <li>
                  <a href="#">
                    <img
                      src="assets/svg/social/behance-1.svg"
                      alt=""
                      className="fn__svg"
                    />
                  </a>
                </li>
              </ul>
            </div>
            <div className="metaportal_fn_breadcrumbs">
              <p>
                <NavLink to="/">
                  <a>Home</a>
                </NavLink>
                <span className="separator">/</span>
                <NavLink to="/collection">
                  <a>Collection</a>
                </NavLink>
                <span className="separator">/</span>
                <span className="current">Meta Legends #5675</span>
              </p>
            </div>
            <h3
              className="fn__maintitle"
              data-text="Meta Legends #5675"
              data-align="left"
            >
              Meta Legends #5675
            </h3>
            <div className="desc">
              <p>
                Suspendisse eu velit est. Cras nec vestibulum quam. Donec
                tincidunt purus nec enim tincidunt, sit amet facilisis massa
                laoreet. Integer mollis nec sapien eu lacinia. Nunc eu massa
                dictum, vulputate neque ac, porta mauris. Sed sollicitudin nisi
                augue, a gravida turpis elementum vel. Curabitur sagittis quis
                diam et rhoncus. Nam pellentesque imperdiet aliquet. Sed non
                ante malesuada, ultrices sem at, tempus libero.
              </p>
              <p>
                Duis eu lorem ut mauris pulvinar auctor. Maecenas et dapibus
                orci, eleifend euismod justo. Quisque luctus turpis eu tristique
                feugiat. Praesent ac magna feugiat, interdum lacus ac, interdum
                dui. Pellentesque id quam quis enim malesuada rutrum. Orci
                varius natoque penatibus et magnis dis parturient.
              </p>
            </div>
            <div className="view_on">
              <ul>
                <li>
                  <span>View On:</span>
                </li>
                <li>
                  <a href="#">
                    <img
                      src="assets/svg/opensea.svg"
                      alt=""
                      className="fn__svg"
                    />
                  </a>
                </li>
                <li>
                  <a href="#">
                    <img
                      src="assets/svg/portal.svg"
                      alt=""
                      className="fn__svg"
                    />
                  </a>
                </li>
              </ul>
            </div>
          </div>
        </div>
        <div className="metaportal_fn_mintbox">
          <div className="mint_left">
            <div className="mint_title">
              <span>Public Mint is Live</span>
            </div>
            <div className="mint_list">
              <ul>
                <li>
                  <div className="item">
                    <h4>Price</h4>
                    <h3>2.25 ETH</h3>
                  </div>
                </li>
                <li>
                  <div className="item">
                    <h4>Remaining</h4>
                    <h3>344/999</h3>
                  </div>
                </li>
                <li>
                  <div className="item">
                    <h4>Quantity</h4>
                    <div className="qnt">
                      <span className="decrease">-</span>
                      <span className="summ" data-price="2.25">
                        2
                      </span>
                      <span className="increase">+</span>
                    </div>
                  </div>
                </li>
                <li>
                  <div className="item">
                    <h4>Total Price</h4>
                    <h3>
                      <span className="total_price">4.5</span> ETH + GAS
                    </h3>
                  </div>
                </li>
              </ul>
            </div>
            <div className="mint_desc">
              <a
                href="#"
                target="_blank"
                rel="noreferrer"
                className="metaportal_fn_button"
              >
                <span>Mint Now</span>
              </a>
              <p>
                By clicking “MINT NOW” button, you agree to our{' '}
                <a href="#">Terms of Service</a> and our{' '}
                <a href="#">Privacy Policy</a>.
              </p>
            </div>
          </div>
          <div className="mint_right">
            <div className="mright">
              <Clock targetDate={targetDate} />
              <div className="img_in " data-bg-img="assets/img/lock.png">
                <img src="assets/img/1x1.jpg" alt="" />
              </div>
            </div>
          </div>
        </div>
        <div className="metaportal_fn_nft_cats">
          <ul>
            <li>
              <div className="item">
                <h4 className="parent_category">Clothing</h4>
                <h3 className="child_category" title="Black Yukata">
                  Black Yukata
                </h3>
              </div>
            </li>
            <li>
              <div className="item">
                <h4 className="parent_category">Eyes</h4>
                <h3 className="child_category" title="Daydreaming">
                  Daydreaming
                </h3>
              </div>
            </li>
            <li>
              <div className="item">
                <h4 className="parent_category">Special</h4>
                <h3 className="child_category" title="Fireflies, Smoke">
                  Fireflies, Smoke
                </h3>
              </div>
            </li>
            <li>
              <div className="item">
                <h4 className="parent_category">Type</h4>
                <h3 className="child_category" title="Human, Sand">
                  Human, Sand
                </h3>
              </div>
            </li>
            <li>
              <div className="item">
                <h4 className="parent_category">Mouth</h4>
                <h3 className="child_category" title="Not Bad">
                  Not Bad
                </h3>
              </div>
            </li>
            <li>
              <div className="item">
                <h4 className="parent_category">Neck</h4>
                <h3 className="child_category" title="Zen Headphones">
                  Zen Headphones
                </h3>
              </div>
            </li>
            <li>
              <div className="item">
                <h4 className="parent_category">Eyes</h4>
                <h3 className="child_category" title="Striking">
                  Striking
                </h3>
              </div>
            </li>
            <li>
              <div className="item">
                <h4 className="parent_category">Neck</h4>
                <h3 className="child_category" title="Zen Headphones">
                  Zen Headphones
                </h3>
              </div>
            </li>
          </ul>
        </div>
      </div> */}
    </div>
  );
};
