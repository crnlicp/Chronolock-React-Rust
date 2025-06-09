import CustomizedSteppers from '../components/create/Stepper';
import { useEffect, useState } from 'react';
import { UnlockTimeAndRecipients } from '../components/create/UnlockTimeAndRecipients';
import { PickerValue } from '@mui/x-date-pickers/internals';
import { UploadFile } from '../components/create/UploadFile';
import { Details } from './Details';

export const Create = () => {
  const [activeStep, setActiveStep] = useState(0);
  const [lockTime, setLockTime] = useState<PickerValue | null>(null);
  const [recipients, setRecipients] = useState<string[]>([]);
  const [_mediaUrl, setMediaUrl] = useState<string | null>(null);
  // const targetDate = new Date();
  // targetDate.setHours(targetDate.getHours() + 2325);

  useEffect(() => {
    window.scrollTo(0, 0);
  }, []);

  const handleNext = () => {
    setActiveStep((prevActiveStep) => prevActiveStep + 1);
  };
  const handleBack = () => {
    setActiveStep((prevActiveStep) => prevActiveStep - 1);
  };

  const handleDateChange = (date: PickerValue | null): void => {
    setLockTime(date);
  };

  const handleRecipientChange = (recipients: string[]): void => {
    setRecipients(recipients);
  };

  const handleMediaUrlChange = (url: string): void => {
    setMediaUrl(url);
  };

  return (
    <div className="container page_container">
      <CustomizedSteppers activeStep={activeStep} />
      {activeStep === 0 && (
        <UnlockTimeAndRecipients
          lockTime={lockTime}
          recipients={recipients}
          onNext={handleNext}
          onDateChange={handleDateChange}
          onRecipientsChange={handleRecipientChange}
        />
      )}
      {activeStep === 1 && (
        <UploadFile
          onNext={handleNext}
          onBack={handleBack}
          onUrlChange={handleMediaUrlChange}
        />
      )}
      {activeStep === 2 && <Details onNext={handleNext} onBack={handleBack} />}
    </div>
  );
};
