import CountUp from 'react-countup';
import ReactVisibilitySensor from 'react-visibility-sensor';
interface ICounterProps {
  end?: number;
  decimals?: number;
}
export const Counter = ({ end, decimals }: ICounterProps) => {
  return (
    <CountUp
      end={end ? end : 100}
      duration={3}
      decimals={decimals ? decimals : 0}
    >
      {({ countUpRef, start }) => (
        <ReactVisibilitySensor onChange={start} delayedCall>
          <span
            data-from="0"
            className="fn_cs_counter"
            data-to={end}
            ref={countUpRef}
          >
            count
          </span>
        </ReactVisibilitySensor>
      )}
    </CountUp>
  );
};
