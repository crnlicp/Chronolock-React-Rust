import { NavLink } from 'react-router';

export const Footer = () => {
  return (
    <footer id="footer">
      <div className="container">
        <div className="footer">
          <div className="left_part">
            <p>Copyright 2025</p>
          </div>
          <div className="right_part">
            <ul>
              <li>
                <NavLink to="/policy">
                  <span className="creative_link">Privacy Policy</span>
                </NavLink>
              </li>
              <li>
                <NavLink to="/cookies">
                  <span className="creative_link">Cookies</span>
                </NavLink>
              </li>
              <li>
                <NavLink to="/terms-conditions">
                  <span className="creative_link">Terms &amp; Conditions</span>
                </NavLink>
              </li>
            </ul>
          </div>
        </div>
      </div>
    </footer>
  );
};
