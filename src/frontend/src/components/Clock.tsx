import React, { useState, useEffect, useRef, RefObject } from 'react';
import '../styles/clock.css';

interface CountdownClockProps {
  targetDate: Date;
  className?: string;
}

const Clock: React.FC<CountdownClockProps> = ({ targetDate, className }) => {
  const [_time, setTime] = useState('');
  const [date, setDate] = useState('');

  const calculateTimeLeft = () => {
    const now = new Date();
    const difference = targetDate.getTime() - now.getTime();

    if (difference <= 0) {
      return {
        days: 0,
        hours: 0,
        minutes: 0,
        seconds: 0,
      };
    }

    return {
      days: Math.floor(difference / (1000 * 60 * 60 * 24)),
      hours: Math.floor((difference / (1000 * 60 * 60)) % 24),
      minutes: Math.floor((difference / 1000 / 60) % 60),
      seconds: Math.floor((difference / 1000) % 60),
    };
  };

  const [daysLeft, setDaysLeft] = useState(calculateTimeLeft().days);

  const secondRef = useRef<HTMLDivElement>(null);
  const minuteRef = useRef<HTMLDivElement>(null);
  const hourRef = useRef<HTMLDivElement>(null);
  const daysRef = useRef<HTMLDivElement>(null);
  const secondDialRef = useRef<HTMLDivElement>(null);
  const minuteDialRef = useRef<HTMLDivElement>(null);
  const dailRef = useRef<HTMLDivElement>(null);
  const hourDialRef = useRef<HTMLDivElement>(null);
  const daysDialRef = useRef<HTMLDivElement>(null);

  const createDial = (
    ref: RefObject<HTMLDivElement>,
    size: number,
    count: number,
    useHours = false,
    start = 0,
  ) => {
    for (
      let s = !!start && start > count / 2 ? start - count / 2 : 0;
      start ? s < start + count / 2 : s < count;
      s++
    ) {
      const span = document.createElement('span');
      const rotation = useHours ? (360 / count) * s : 6 * s;
      span.style.transform = `rotate(${rotation}deg) translateX(${
        size * 0.51
      }cqw)`;
      span.textContent = s.toString();
      ref.current?.appendChild(span);
    }
  };

  const updateTime = () => {
    const timeLeft = calculateTimeLeft();

    // Update analog clock hands
    if (secondRef.current) {
      secondRef.current.style.transform = `rotate(${timeLeft.seconds * -6}deg)`;
    }
    if (minuteRef.current) {
      minuteRef.current.style.transform = `rotate(${timeLeft.minutes * -6}deg)`;
    }
    if (hourRef.current) {
      hourRef.current.style.transform = `rotate(${timeLeft.hours * -15}deg)`;
    }
    if (daysRef.current) {
      daysRef.current.style.transform = `rotate(${timeLeft.days * -15}deg)`;
    }

    setTime(
      `${timeLeft.days.toString().padStart(2, '0')}:${timeLeft.hours
        .toString()
        .padStart(2, '0')}:${timeLeft.minutes
        .toString()
        .padStart(2, '0')}:${timeLeft.seconds.toString().padStart(2, '0')}`,
    );
    setDate(targetDate.toLocaleString());
    setDaysLeft(timeLeft.days);
  };

  useEffect(() => {
    setDaysLeft(calculateTimeLeft().days);
  }, [targetDate]);

  useEffect(() => {
    updateTime();
    createDial(secondDialRef, 85, 60);
    createDial(minuteDialRef, 69, 60);
    createDial(hourDialRef, 49, 24, true);
    createDial(daysDialRef, 28, 24, true, daysLeft);
    createDial(dailRef, 95, 60);

    const interval = setInterval(updateTime, 1000);
    return () => clearInterval(interval);
  }, [targetDate]);

  return (
    <div className={'lock-container' + (className ? ` ${className}` : '')}>
      {/* <img src="assets/img/lock.png" className="lock" /> */}
      <div className="clock-container">
        <div className="clock-digital">
          <div className="round-border"></div>
          <div className="seconds">Seconds</div>
          <div className="minutes">Minutes</div>
          <div className="hours">Hours</div>
          <div className="days">Days</div>
          <div className="unlock">Until Unlock</div>
          <div className="target-day">{date}</div>
        </div>
        <div className="clock-analog">
          <div className="spear"></div>
          <div ref={daysRef} className="days">
            <div ref={daysDialRef}></div>
          </div>
          <div ref={hourRef} className="hour">
            <div ref={hourDialRef}></div>
          </div>
          <div ref={minuteRef} className="minute">
            <div ref={minuteDialRef}></div>
          </div>
          <div ref={secondRef} className="second">
            <div ref={secondDialRef}></div>
          </div>
          <div ref={dailRef} className="dail"></div>
        </div>
      </div>
    </div>
  );
};

export default Clock;
