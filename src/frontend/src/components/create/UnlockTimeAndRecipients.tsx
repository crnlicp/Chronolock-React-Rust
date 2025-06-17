import { renderTimeViewClock, StaticDateTimePicker } from '@mui/x-date-pickers';
import { PickerValue } from '@mui/x-date-pickers/internals';
import { useState } from 'react';

interface IUnlockTimeAndRecipientsProps {
  lockTime: number | null;
  recipients: string[];
  onDateChange: (date: number | null) => void;
  onRecipientsChange: (recipients: string[]) => void;
  onNext: () => void;
}

export const UnlockTimeAndRecipients = ({
  lockTime,
  recipients,
  onNext,
  onDateChange,
  onRecipientsChange,
}: IUnlockTimeAndRecipientsProps) => {
  const [picker, setPicker] = useState<PickerValue | null>(null);

  const handleDateChange = (newValue: any) => {
    setPicker(newValue);
    if (newValue) {
      const unixTimestamp = Math.floor(new Date(newValue).getTime() / 1000); // Convert to seconds
      console.log('Unix Timestamp:', unixTimestamp); // Log the Unix timestamp
      onDateChange(unixTimestamp);
    }
  };

  const handleAddRecipient = () => {
    onRecipientsChange([...recipients, '']);
  };

  const handleRecipientChange = (index: number, value: string) => {
    const newRecipients = [...recipients];
    newRecipients[index] = value;
    onRecipientsChange(newRecipients);
  };

  const handleRemoveRecipient = (index: number) => {
    const newRecipients = [...recipients];
    newRecipients.splice(index, 1);
    onRecipientsChange(newRecipients);
  };

  return (
    <div className="container small">
      <div className="fn_cs_contact_form">
        <ul style={{}}>
          <li>
            <input
              id="name"
              type="text"
              placeholder="Unlock time*"
              value={lockTime?.toString().slice(0, 25) ?? ''}
              style={{ fontSize: '24px' }}
              readOnly
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
                minHeight: '580px',
              }}
              value={picker ?? null}
              onChange={handleDateChange}
            />
          </li>
          <li className="unlock_time_reciepients">
            <div
              style={{
                display: 'flex',
                justifyContent: 'center',
                cursor: 'pointer',
                zIndex: 1,
                marginBottom: 24,
              }}
              className="metaportal_fn_button full"
              onClick={handleAddRecipient}
            >
              <span>Add Recipient Principal</span>
            </div>
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
                    id={`recipient-${index}`}
                    type="text"
                    placeholder="Reciepient Principal *"
                    style={{ marginBottom: 16, width: '90%' }}
                    value={recipient}
                    onChange={(e) =>
                      handleRecipientChange(index, e.target.value)
                    }
                  />
                </div>

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
              </div>
            ))}
          </li>
        </ul>
      </div>
      <button
        className="metaportal_fn_button full cursor"
        disabled={!lockTime || recipients.some((r) => r === '')}
        style={{ border: 'none', marginBottom: 24, zIndex: 1 }}
        onClick={onNext}
      >
        <span>Next</span>
      </button>
    </div>
  );
};
