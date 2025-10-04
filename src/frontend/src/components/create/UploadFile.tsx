import { useEffect, useMemo, useState } from 'react';
import { useDropzone } from 'react-dropzone';
import { FileWithPreview } from '../../pages/Create';
import { useChronolock } from '../../hooks/useChronolock';
import { useCrnlToken } from '../../hooks/useCrnlToken';
import {
  Box,
  Button,
  CircularProgress,
  Dialog,
  DialogActions,
  DialogContent,
  DialogContentText,
  DialogTitle,
  IconButton,
} from '@mui/material';
import { Close } from '@mui/icons-material';

const MEDIA_CHRONOLOCK_COST = 20n * 10n ** 8n;

const baseStyle: React.CSSProperties = {
  flex: 1,
  display: 'flex',
  flexDirection: 'column',
  alignItems: 'center',
  padding: '100px',
  borderWidth: 3,
  borderRadius: 10,
  borderColor: '#eeeeee',
  borderStyle: 'dashed',
  color: '#bdbdbd',
  outline: 'none',
  maxWidth: '100%',
  height: '300px',
  transition: 'border .24s ease-in-out',
};

const focusedStyle: React.CSSProperties = { borderColor: '#2196f3' };
const acceptStyle: React.CSSProperties = { borderColor: '#00e676' };
const rejectStyle: React.CSSProperties = { borderColor: '#ff1744' };

const thumbsContainer: React.CSSProperties = {
  display: 'flex',
  flexDirection: 'row',
  flexWrap: 'wrap',
};

const thumb: React.CSSProperties = {
  display: 'inline-flex',
  borderRadius: 10,
  border: '1px solid #eaeaea',
  marginBottom: 8,
  marginRight: 8,
  width: '100%',
  justifyContent: 'center',
  maxWidth: '100%',
  height: '300px',
  padding: 5,
  aspectRatio: '1 / 1',
  boxSizing: 'border-box',
};

const thumbInner: React.CSSProperties = {
  display: 'flex',
  flexDirection: 'column',
  justifyContent: 'center',
  alignItems: 'center',
  minWidth: 0,
  overflow: 'hidden',
};

const img: React.CSSProperties = {
  display: 'block',
  width: 'auto',
  height: '85%',
};

interface IUploadFileProps {
  files: FileWithPreview[];
  mediaId: string | undefined;
  cryptoKey: CryptoKey | undefined;
  setFiles: React.Dispatch<React.SetStateAction<FileWithPreview[]>>;
  setFileType: React.Dispatch<React.SetStateAction<string | undefined>>;
  setMediaSize: React.Dispatch<React.SetStateAction<number | undefined>>;
  onNext: () => void;
  onBack: () => void;
  onSetMediaId: (mediaId: string) => void;
}

export const UploadFile = ({
  files,
  mediaId,
  cryptoKey,
  setFiles,
  setFileType,
  setMediaSize,
  onNext,
  onBack,
  onSetMediaId,
}: IUploadFileProps) => {
  const { upload, isUploadLoading, uploadErrors } = useChronolock();
  const {
    balanceData,
    balanceRaw,
    checkBalance,
    deductFromBalance,
    isDeductFromBalanceLoading,
    deductFromBalanceError,
  } = useCrnlToken();
  const [error, setError] = useState<string | null>(null);
  const [isConfirmOpen, setIsConfirmOpen] = useState(false);
  const [confirmError, setConfirmError] = useState<string | null>(null);

  const { getRootProps, getInputProps, isFocused, isDragAccept, isDragReject } =
    useDropzone({
      onDrop: (acceptedFiles, fileRejections) => {
        setError(null);
        setFiles(
          acceptedFiles.map((file) => ({
            file,
            preview: URL.createObjectURL(file),
          })),
        );
        if (acceptedFiles.length > 0) {
          const fileType = acceptedFiles[0].type;
          setFileType(fileType);
        } else {
          setFileType(undefined);
        }
        fileRejections.forEach(({ errors }) => {
          errors.forEach(({ message }) => {
            setError(message);
          });
        });
      },
      maxFiles: 1,
      maxSize: 10 * 1024 * 1024, // 10 MB
      multiple: false,
    });

  useEffect(() => {
    const filesWithPreview = files.map((file) => ({
      ...file,
      preview: URL.createObjectURL(file.file),
    }));
    setFiles(filesWithPreview);
    setFileType(filesWithPreview[0]?.file.type);
    return () => {
      files.forEach(({ preview }) => {
        if (preview) {
          URL.revokeObjectURL(preview);
        }
      });
    };
  }, []);

  const style = useMemo(
    () => ({
      ...baseStyle,
      ...(isFocused ? focusedStyle : {}),
      ...(isDragAccept ? acceptStyle : {}),
      ...(isDragReject ? rejectStyle : {}),
    }),
    [isFocused, isDragAccept, isDragReject],
  );

  const thumbs = files.map((file) => (
    <div style={thumb} key={file.file.name}>
      <div style={thumbInner}>
        {file.file.type.startsWith('image/') ? (
          <img src={file.preview} style={img} alt={file.file.name} />
        ) : file.file.type.startsWith('video/') ? (
          <video src={file.preview} style={img} controls />
        ) : (
          <div style={{ fontSize: 70, textAlign: 'center', width: '100%' }}>
            📄
          </div>
        )}
        <div
          style={{
            fontSize: 20,
            textAlign: 'center',
            width: '100%',
            marginTop: 10,
          }}
        >
          {file.file.name}
        </div>
      </div>
    </div>
  ));

  const hasSufficientBalance = balanceRaw >= MEDIA_CHRONOLOCK_COST;

  const handleOpenConfirm = () => {
    if (files.length === 0 || !cryptoKey) {
      const message =
        'Some error occurred: No files selected or crypto key is missing';
      setError(message);
      return;
    }
    if (!hasSufficientBalance) {
      const message = `Insufficient balance. You need at least 20 $CRNL to upload media. Current balance: ${balanceData} $CRNL.`;
      setError(message);
      return;
    }
    setError(null);
    setConfirmError(null);
    setIsConfirmOpen(true);
  };

  const handleCloseConfirm = () => {
    if (isDeductFromBalanceLoading || isUploadLoading) {
      return;
    }
    setIsConfirmOpen(false);
    setConfirmError(null);
  };

  const performUpload = async () => {
    if (files.length === 0 || !cryptoKey) {
      throw new Error(
        'Some error occurred: No files selected or crypto key is missing',
      );
    }

    const arrayBuffer = await files[0].file.arrayBuffer();
    const iv = window.crypto.getRandomValues(new Uint8Array(12));
    const encryptedBuffer = await window.crypto.subtle.encrypt(
      { name: 'AES-GCM', iv },
      cryptoKey,
      arrayBuffer,
    );
    const concatenatedArray = new Uint8Array(
      iv.length + encryptedBuffer.byteLength,
    );
    concatenatedArray.set(iv, 0);
    concatenatedArray.set(new Uint8Array(encryptedBuffer), iv.length);
    const concatenatedBuffer = concatenatedArray.buffer;

    const result = await upload(concatenatedBuffer);

    if (
      result &&
      typeof result === 'object' &&
      'urlObject' in result &&
      'mediaId' in result &&
      typeof result.urlObject === 'object' &&
      'Ok' in (result.urlObject as { Ok: string }) &&
      typeof result.mediaId === 'string'
    ) {
      onSetMediaId(result.mediaId as string);
      setMediaSize(files[0].file.size);
      return;
    }

    throw new Error('Upload failed: Unexpected response');
  };

  const handleConfirmUpload = async () => {
    if (files.length === 0 || !cryptoKey) {
      const message =
        'Some error occurred: No files selected or crypto key is missing';
      setError(message);
      setConfirmError(message);
      setIsConfirmOpen(false);
      return;
    }
    if (!hasSufficientBalance) {
      const message = `Insufficient balance. You need at least 20 $CRNL to upload media. Current balance: ${balanceData} $CRNL.`;
      setError(message);
      setConfirmError(message);
      setIsConfirmOpen(false);
      return;
    }

    try {
      setConfirmError(null);
      setError(null);
      await deductFromBalance(
        MEDIA_CHRONOLOCK_COST,
        'Media Chronolock upload fee',
      );
      await performUpload();
      await checkBalance();
      setIsConfirmOpen(false);
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Upload failed to complete';
      setConfirmError(message);
      setError(message);
    }
  };

  function handleRemoveFile(
    event: React.MouseEvent<HTMLButtonElement, MouseEvent>,
  ): void {
    event.preventDefault();
    event.stopPropagation();

    // Revoke object URLs to avoid memory leaks
    files.forEach(({ preview }) => {
      if (preview) {
        URL.revokeObjectURL(preview);
      }
    });

    // Clear file-related state
    setFiles([]);
    setFileType(undefined);
    setMediaSize(undefined);

    // Clear any errors and confirmation state
    setError(null);
    setConfirmError(null);

    // Clear mediaId (callback expects a string, use empty string to indicate cleared)
    if (mediaId) {
      onSetMediaId('');
    }
  }
  return (
    <div className="container small">
      <div className="fn_cs_contact_form">
        <ul>
          <li>
            <section>
              <div {...getRootProps({ style })}>
                <input
                  {...getInputProps()}
                  disabled={isUploadLoading || isDeductFromBalanceLoading}
                />
                <p>Drag 'n' drop some files here, or click to select files</p>
              </div>
            </section>
          </li>
          <li>
            <div>
              {error && (
                <p style={{ color: 'red', marginTop: '10px' }}>{error}</p>
              )}
              <aside style={thumbsContainer}>
                {thumbs}
                {files && files.length > 0 && (
                  <IconButton
                    sx={{ position: 'absolute' }}
                    onClick={handleRemoveFile}
                  >
                    <Close color="error" fontSize="large" />
                  </IconButton>
                )}
              </aside>
            </div>
          </li>
        </ul>

        {!!files.length && !mediaId && (
          <button
            className="metaportal_fn_button full cursor"
            disabled={
              files.length === 0 ||
              isUploadLoading ||
              isDeductFromBalanceLoading ||
              !hasSufficientBalance
            }
            style={{
              border: 'none',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              cursor:
                isUploadLoading ||
                isDeductFromBalanceLoading ||
                !hasSufficientBalance
                  ? 'not-allowed'
                  : 'pointer',
            }}
            onClick={handleOpenConfirm}
          >
            <Box
              mx={2}
              display="flex"
              alignItems="center"
              justifyContent="center"
              width={100}
              position="relative"
            >
              <span>Upload</span>
              {(isUploadLoading ||
                isDeductFromBalanceLoading ||
                !hasSufficientBalance) && (
                <Box
                  display={'flex'}
                  position="absolute"
                  left="100%"
                  top="50%"
                  sx={{ transform: 'translate(-50%, -50%)' }}
                >
                  <CircularProgress size={24} />
                </Box>
              )}
            </Box>
          </button>
        )}
        {(uploadErrors.length > 0 || deductFromBalanceError) && (
          <Box
            my={2}
            display={'flex'}
            flexDirection={'column'}
            justifyContent="center"
            alignItems="center"
          >
            {uploadErrors.map((err, index) => (
              <p
                key={index}
                style={{
                  color: 'red',
                  margin: 0,
                  textAlign: 'left',
                }}
              >
                {err?.message}
              </p>
            ))}
            {deductFromBalanceError && (
              <p
                style={{
                  color: 'red',
                  margin: 0,
                  textAlign: 'left',
                }}
              >
                {deductFromBalanceError.message}
              </p>
            )}
          </Box>
        )}
        {!hasSufficientBalance && (
          <Box
            my={2}
            display={'flex'}
            flexDirection="column"
            justifyContent="center"
            alignItems="center"
          >
            <p
              style={{
                color: 'red',
                margin: 0,
                textAlign: 'center',
              }}
            >
              You need at least 20 $CRNL to upload media. Current balance:{' '}
              {balanceData} $CRNL.
            </p>
          </Box>
        )}
        <Box
          sx={{
            backgroundColor: '#f0f0f0',
            padding: '24px',
            borderRadius: '8px',
          }}
          my={2}
        >
          <h5 style={{ color: 'green', margin: '0', lineHeight: '1.5' }}>
            Note: Changing the file requires re-uploading. Files are encrypted
            and securely uploaded. Please verify the file's accuracy before
            proceeding. You can skip this step if you want to create Text
            Chronolock. Uploading each media file will cost you 20 CRNL. Make
            sure you have enough CRNL in your account
          </h5>
        </Box>
        <ul style={{ marginTop: '100px' }}>
          <li>
            <button
              className="metaportal_fn_button full cursor"
              style={{
                border: 'none',
                zIndex: 1,
              }}
              disabled={isUploadLoading}
              onClick={onBack}
            >
              <span>Back</span>
            </button>
          </li>
          <li>
            <button
              className="metaportal_fn_button full cursor"
              style={{
                border: 'none',
                zIndex: 1,
                marginBottom: 24,
              }}
              disabled={isUploadLoading || (!!files.length && !mediaId)}
              onClick={onNext}
            >
              <span>Next</span>
            </button>
          </li>
        </ul>
      </div>
      <Dialog
        open={isConfirmOpen}
        onClose={handleCloseConfirm}
        disableScrollLock
      >
        <DialogTitle>Confirm Upload Fee</DialogTitle>
        <DialogContent>
          <DialogContentText>
            Uploading media will deduct 20 $CRNL from your balance. This
            deduction is non-refundable even if you re-upload the file later. Do
            you want to continue?
          </DialogContentText>
          <DialogContentText color="success" variant="caption" mt={2}>
            Available Balance: {balanceData} CRNL
          </DialogContentText>
          {confirmError && (
            <DialogContentText color="error" sx={{ mt: 2 }}>
              {confirmError}
            </DialogContentText>
          )}
        </DialogContent>
        <DialogActions>
          <Button
            onClick={handleCloseConfirm}
            disabled={isDeductFromBalanceLoading || isUploadLoading}
          >
            Cancel
          </Button>
          <Button
            onClick={handleConfirmUpload}
            disabled={isDeductFromBalanceLoading || isUploadLoading}
          >
            {isDeductFromBalanceLoading || isUploadLoading ? (
              <CircularProgress size={20} />
            ) : (
              'Confirm'
            )}
          </Button>
        </DialogActions>
      </Dialog>
    </div>
  );
};
