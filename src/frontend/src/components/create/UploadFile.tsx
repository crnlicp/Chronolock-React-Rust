import { useEffect, useMemo, useState } from 'react';
import { useDropzone } from 'react-dropzone';
import { FileWithPreview } from '../../pages/Create';
import { useChronolock } from '../../hooks/useChronolock';
import { Box, CircularProgress } from '@mui/material';

interface IUploadFileProps {
  files: FileWithPreview[];
  mediaUrl: string | null;
  cryptoKey: CryptoKey | null;
  setFiles: React.Dispatch<React.SetStateAction<FileWithPreview[]>>;
  onNext: () => void;
  onBack: () => void;
  onSetMediaId: (mediaId: string) => void;
  onUrlChange: (url: string) => void;
}

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

export const UploadFile = ({
  files,
  mediaUrl,
  cryptoKey,
  setFiles,
  onNext,
  onBack,
  onSetMediaId,
  onUrlChange: onUrlChange,
}: IUploadFileProps) => {
  const { upload, isUploadLoading, uploadErrors } = useChronolock();
  const [error, setError] = useState<string | null>(null);

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
    return () => {
      files.forEach(({ preview }) => URL.revokeObjectURL(preview));
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

  const handleUploadFile = async () => {
    if (files.length === 0 || !cryptoKey) {
      setError(
        'Some error occurred: No files selected or crypto key is missing',
      );
      return;
    }
    const arrayBuffer = await files[0].file.arrayBuffer();
    const iv = window.crypto.getRandomValues(new Uint8Array(12));
    const encryptedBuffer = await window.crypto.subtle.encrypt(
      { name: 'AES-GCM', iv },
      cryptoKey,
      arrayBuffer,
    );
    console.log('file encrypted', {
      iv: iv,
      encryptedBuffer: Array.from(new Uint8Array(encryptedBuffer)),
    });

    const result = await upload(encryptedBuffer);
    if (
      result &&
      typeof result === 'object' &&
      'urlObject' in result &&
      'mediaId' in result &&
      typeof result.urlObject === 'object' &&
      'Ok' in (result.urlObject as { Ok: string }) &&
      typeof result.mediaId === 'string'
    ) {
      const mediaUrl = (result.urlObject as { Ok: string }).Ok;
      console.log('File uploaded successfully:', { Ok: mediaUrl });
      onUrlChange(mediaUrl as string);
      onSetMediaId(result.mediaId as string);

      // Use chunked download
      // const totalSize = files[0].file.size;
      // const media = await getMediaChunked(result.mediaId as string, totalSize);
      // console.log('Media data retrieved (chunked)', { Ok: media });
    } else {
      setError('Upload failed: Unexpected response');
    }
  };

  return (
    <div className="container small">
      <div className="fn_cs_contact_form">
        <ul>
          <li>
            <section>
              <div {...getRootProps({ style })}>
                <input {...getInputProps()} disabled={isUploadLoading} />
                <p>Drag 'n' drop some files here, or click to select files</p>
              </div>
            </section>
          </li>
          <li>
            <div>
              {error && (
                <p style={{ color: 'red', marginTop: '10px' }}>{error}</p>
              )}
              <aside style={thumbsContainer}>{thumbs}</aside>
            </div>
          </li>
        </ul>

        {!!files.length && (
          <button
            className="metaportal_fn_button full cursor"
            disabled={files.length === 0 || isUploadLoading}
            style={{
              border: 'none',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              cursor: isUploadLoading ? 'not-allowed' : 'pointer',
            }}
            onClick={handleUploadFile}
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
              {isUploadLoading && (
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
        {uploadErrors.length > 0 && (
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
          </Box>
        )}
        <ul style={{ marginTop: '200px' }}>
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
              disabled={files.length === 0 || isUploadLoading || !mediaUrl}
              onClick={onNext}
            >
              <span>Next</span>
            </button>
          </li>
        </ul>
      </div>
    </div>
  );
};
