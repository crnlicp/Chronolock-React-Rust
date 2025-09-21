import CountUp from 'react-countup';
import { useInView } from 'react-intersection-observer';
interface ICounterProps {
  end?: number;
  decimals?: number;
}
export const Counter = ({ end, decimals }: ICounterProps) => {
  const { ref, inView } = useInView({ triggerOnce: false });

  return (
    <CountUp
      end={end !== undefined ? end : 0}
      duration={3}
      decimals={decimals !== undefined ? decimals : 0}
      start={inView ? undefined : 0}
    >
      {({ countUpRef }) => (
        <span
          data-from="0"
          className="fn_cs_counter"
          data-to={end}
          ref={(el) => {
            if (typeof ref === 'function') ref(el);
            else if (ref)
              (ref as React.MutableRefObject<HTMLElement | null>).current = el;
            if (countUpRef)
              (
                countUpRef as React.MutableRefObject<HTMLElement | null>
              ).current = el;
          }}
        >
          count
        </span>
      )}
    </CountUp>
  );
};
