import { Swiper, SwiperSlide } from 'swiper/react';
import { roadMapProps } from '../utils/sliderProps';

export const RoadMapSlider = () => {
  return (
    <section id="roadmap">
      <div className="container">
        <h3
          className="fn__maintitle big"
          data-text="RoadMap"
          data-align="center"
        >
          RoadMap
        </h3>
        <div className="fn_cs_roadmap">
          <div className="step_holder">
            <div className="step_in" />
          </div>
          <div className="slider_holder">
            <Swiper {...roadMapProps} className="swiper-container">
              <div className="swiper-wrapper">
                <SwiperSlide className="swiper-slide">
                  <div className="item">
                    <span className="icon" />
                    <span className="phase">Phase 01</span>
                    <div className="item_in">
                      <p className="date">Jan 2025</p>
                      <h3 className="title">Project Kickoff</h3>
                      <p className="desc">
                        Begin with team formation, requirements gathering, and
                        initial planning. Define the project scope, objectives,
                        and success criteria. Establish communication channels
                        and set up the development environment.
                      </p>
                    </div>
                  </div>
                </SwiperSlide>
                <SwiperSlide className="swiper-slide">
                  <div className="item">
                    <span className="icon" />
                    <span className="phase">Phase 02</span>
                    <div className="item_in">
                      <p className="date">Feb 2025</p>
                      <h3 className="title">Architecture & Design</h3>
                      <p className="desc">
                        Design the system architecture, including backend
                        canisters, frontend structure, and integration points.
                        Create wireframes, technical documentation, and select
                        core technologies. Ensure scalability and security are
                        considered from the start.
                      </p>
                    </div>
                  </div>
                </SwiperSlide>
                <SwiperSlide className="swiper-slide">
                  <div className="item">
                    <span className="icon" />
                    <span className="phase">Phase 03</span>
                    <div className="item_in">
                      <p className="date">March 2025</p>
                      <h3 className="title">Core Backend Development</h3>
                      <p className="desc">
                        Develop the main backend canisters in Rust, focusing on
                        essential business logic, data models, and APIs.
                        Implement initial smart contract functionality and set
                        up local testing environments.
                      </p>
                    </div>
                  </div>
                </SwiperSlide>
                <SwiperSlide className="swiper-slide">
                  <div className="item">
                    <span className="icon" />
                    <span className="phase">Phase 04</span>
                    <div className="item_in">
                      <p className="date">Aug 2025</p>
                      <h3 className="title">Frontend MVP</h3>
                      <p className="desc">
                        Build a minimum viable product (MVP) for the frontend
                        using React and TypeScript. Integrate basic UI
                        components, connect to backend canisters, and enable
                        core user flows for early feedback.
                      </p>
                    </div>
                  </div>
                </SwiperSlide>
                <SwiperSlide className="swiper-slide">
                  <div className="item">
                    <span className="icon" />
                    <span className="phase">Phase 05</span>
                    <div className="item_in">
                      <p className="date">Sep 2025</p>
                      <h3 className="title">Identity & Authentication</h3>
                      <p className="desc">
                        Implement robust identity management and authentication
                        mechanisms. Integrate with Internet Identity to ensure
                        user data protection and seamless onboarding.
                      </p>
                    </div>
                  </div>
                </SwiperSlide>
                <SwiperSlide className="swiper-slide">
                  <div className="item">
                    <span className="icon" />
                    <span className="phase">Phase 06</span>
                    <div className="item_in">
                      <p className="date">Future plan</p>
                      <h3 className="title">Feature Expansion</h3>
                      <p className="desc">
                        Add advanced features and additional canister
                        interactions. Enhance the frontend with new pages,
                        improved UX, and responsive design.
                      </p>
                    </div>
                  </div>
                </SwiperSlide>
                <SwiperSlide className="swiper-slide">
                  <div className="item">
                    <span className="icon" />
                    <span className="phase">Phase 07</span>
                    <div className="item_in">
                      <p className="date">Future plan</p>
                      <h3 className="title">Testing & QA</h3>
                      <p className="desc">
                        Conduct thorough unit, integration, and end-to-end
                        testing across backend and frontend. Address bugs,
                        optimize performance, and ensure reliability. Prepare
                        for external beta testing.
                      </p>
                    </div>
                  </div>
                </SwiperSlide>
                <SwiperSlide className="swiper-slide">
                  <div className="item">
                    <span className="icon" />
                    <span className="phase">Phase 08</span>
                    <div className="item_in">
                      <p className="date">Future plan</p>
                      <h3 className="title">Beta Launch</h3>
                      <p className="desc">
                        Release a beta version to a select group of users.
                        Gather feedback, monitor system performance, and iterate
                        on features and UI based on real-world usage.
                      </p>
                    </div>
                  </div>
                </SwiperSlide>
                <SwiperSlide className="swiper-slide">
                  <div className="item">
                    <span className="icon" />
                    <span className="phase">Phase 09</span>
                    <div className="item_in">
                      <p className="date">Future plan</p>
                      <h3 className="title">Security Audit & Optimization</h3>
                      <p className="desc">
                        Perform comprehensive security audits of smart contracts
                        and application code. Optimize for scalability,
                        cost-efficiency, and robustness. Address any
                        vulnerabilities and finalize documentation.
                      </p>
                    </div>
                  </div>
                </SwiperSlide>
                <SwiperSlide className="swiper-slide">
                  <div className="item">
                    <span className="icon" />
                    <span className="phase">Phase 10</span>
                    <div className="item_in">
                      <p className="date">Future plan</p>
                      <h3 className="title">Production Release</h3>
                      <p className="desc">
                        Launch the fully functional, production-ready product.
                        Deploy on mainnet, provide user support, and initiate
                        marketing campaigns. Continue monitoring, maintenance,
                        and plan for future updates.
                      </p>
                    </div>
                  </div>
                </SwiperSlide>
              </div>
            </Swiper>
          </div>
        </div>
      </div>
    </section>
  );
};
