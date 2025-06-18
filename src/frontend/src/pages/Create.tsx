import CustomizedSteppers from '../components/create/Stepper';
import { useEffect, useState } from 'react';
import { UnlockTimeAndRecipients } from '../components/create/UnlockTimeAndRecipients';
import { UploadFile } from '../components/create/UploadFile';
import { Details } from '../components/create/Details';
import { useAuth } from '../hooks/useAuth';
import { useChronolock } from '../hooks/useChronolock';
import ReviewAndCreate from '../components/create/ReviewAndCreate';

export type FileWithPreview = { file: File; preview: string };

export const Create = () => {
  const { isAuthenticated } = useAuth();
  const { generateKey } = useChronolock();

  const [name, setName] = useState<string>('');
  const [title, setTitle] = useState<string>('');
  const [description, setDescription] = useState<string>('');
  const [attributes, setAttributes] = useState<
    { key: string; value: string }[]
  >([]);
  const [cryptoKey, setCryptoKey] = useState<CryptoKey | undefined>(undefined);
  const [activeStep, setActiveStep] = useState(0);
  const [lockTime, setLockTime] = useState<number | undefined>(undefined);
  const [recipients, setRecipients] = useState<string[]>([]);
  const [files, setFiles] = useState<FileWithPreview[]>([]);
  const [fileType, setFileType] = useState<string | undefined>(undefined);
  const [mediaUrl, setMediaUrl] = useState<string | undefined>(undefined);
  const [mediaId, setMediaId] = useState<string | undefined>(undefined);

  useEffect(() => {
    window.scrollTo(0, 0);
    generateKey()
      .then((generatedKey) => {
        setCryptoKey(generatedKey);
      })
      .catch((error) => {
        console.error('Error generating key:', error);
      });
  }, []);

  const handleNext = () => {
    setActiveStep((prevActiveStep) => prevActiveStep + 1);
  };
  const handleBack = () => {
    setActiveStep((prevActiveStep) => prevActiveStep - 1);
  };

  const handleDateChange = (date: number | undefined): void => {
    setLockTime(date);
  };

  const handleRecipientChange = (recipients: string[]): void => {
    setRecipients(recipients);
  };

  const handleMediaUrlChange = (url: string): void => {
    setMediaUrl(url);
  };

  const handleChangeName = (name: string): void => {
    setName(name);
  };

  const handleChangeTitle = (title: string): void => {
    setTitle(title);
  };

  const handleChangeDescription = (description: string): void => {
    setDescription(description);
  };

  const handleSetMediaId = (id: string): void => {
    setMediaId(id);
  };

  if (!isAuthenticated) {
    return (
      <div
        className="container page_container"
        style={{ textAlign: 'center', marginTop: '50px' }}
      >
        <h2>Please log in to Continue</h2>
      </div>
    );
  }

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
          files={files}
          mediaUrl={mediaUrl}
          cryptoKey={cryptoKey}
          setFiles={setFiles}
          setFileType={setFileType}
          onSetMediaId={handleSetMediaId}
          onNext={handleNext}
          onBack={handleBack}
          onUrlChange={handleMediaUrlChange}
        />
      )}
      {activeStep === 2 && (
        <Details
          name={name}
          title={title}
          description={description}
          attributes={attributes}
          setAttributes={setAttributes}
          onChangeName={handleChangeName}
          onChangeTitle={handleChangeTitle}
          onChangeDescription={handleChangeDescription}
          onBack={handleBack}
          onNext={handleNext}
        />
      )}
      {activeStep === 3 && (
        <ReviewAndCreate
          name={name}
          title={title}
          description={description}
          attributes={attributes}
          mediaUrl={mediaUrl}
          fileType={fileType}
          lockTime={lockTime}
          recipients={recipients}
          cryptoKey={cryptoKey}
          mediaId={mediaId}
          onBack={handleBack}
        />
      )}
    </div>
  );
};
