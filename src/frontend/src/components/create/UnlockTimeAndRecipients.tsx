import { renderTimeViewClock, StaticDateTimePicker } from '@mui/x-date-pickers';
import { useState } from 'react';

interface IUnlockTimeAndRecipientsProps {
  onDateChange: (date: string | null) => void;
  onRecipientsChange: (recipients: string[]) => void;
  onNext: () => void;
}

export const UnlockTimeAndRecipients = ({
  onNext,
}: IUnlockTimeAndRecipientsProps) => {
  const [timeValue, setTimeValue] = useState<string | null>();
  const [recipients, setRecipients] = useState<string[]>(['']);

  const handleDateChange = (newValue: any) => {
    setTimeValue(newValue);
  };

  const handleAddRecipient = () => {
    setRecipients([...recipients, '']);
  };

  const handleRecipientChange = (index: number, value: string) => {
    const newRecipients = [...recipients];
    newRecipients[index] = value;
    setRecipients(newRecipients);
  };

  const handleRemoveRecipient = (index: number) => {
    const newRecipients = [...recipients];
    newRecipients.splice(index, 1);
    setRecipients(newRecipients);
  };

  return (
    <div className="container small">
      <div className="fn_cs_contact_form">
        <div className="input_list">
          <ul>
            <li>
              <input
                id="name"
                type="text"
                placeholder="Unlock time*"
                value={timeValue?.toString().slice(0, 25)}
                style={{ fontSize: '24px' }}
              />
              <StaticDateTimePicker
                disablePast
                ampm={false}
                orientation="landscape"
                views={['year', 'month', 'day', 'hours', 'minutes', 'seconds']}
                viewRenderers={{
                  hours: renderTimeViewClock,
                  minutes: renderTimeViewClock,
                  seconds: renderTimeViewClock,
                }}
                sx={{
                  marginY: 2,
                  backgroundColor: 'lightsteelblue',
                  color: 'black',
                }}
                onChange={handleDateChange}
              />
            </li>
            <li className="unlock_time_reciepients">
              {recipients.map((recipient, index) => (
                <div
                  style={{ display: 'flex', justifyContent: 'space-between' }}
                  key={index}
                >
                  <div
                    style={{
                      width: '100%',
                      height: '100%',
                    }}
                  >
                    <input
                      id="name"
                      type="text"
                      placeholder="Reciepient Principal *"
                      style={{ marginBottom: 16, width: '90%' }}
                      value={recipient}
                      onChange={(e) =>
                        handleRecipientChange(index, e.target.value)
                      }
                    />
                  </div>
                  {index !== 0 && (
                    <div
                      className="metaportal_fn_button full"
                      style={{
                        marginLeft: 16,
                        width: '15%',
                        fontSize: '24px',
                        cursor: 'pointer',
                      }}
                      onClick={() => handleRemoveRecipient(index)}
                    >
                      -
                    </div>
                  )}
                </div>
              ))}
              <div
                style={{
                  display: 'flex',
                  justifyContent: 'center',
                  cursor: 'pointer',
                  zIndex: 1,
                }}
                className="metaportal_fn_button full margin_button"
                onClick={handleAddRecipient}
              >
                <span>Add More</span>
              </div>
            </li>
          </ul>
          <button
            className="metaportal_fn_button full"
            disabled={!timeValue || recipients.some((r) => r === '')}
            style={{ marginBottom: '50px', cursor: 'pointer', border: 'none' }}
            onClick={onNext}
          >
            <span>Next</span>
          </button>
        </div>
      </div>
    </div>
  );
};
