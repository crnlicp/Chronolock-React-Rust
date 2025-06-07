import { useEffect, useMemo, useState } from 'react';
import { useDropzone } from 'react-dropzone';

interface IUploadFileProps {
  onNext: () => void;
  onBack: () => void;
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
  onNext,
  onBack,
  onUrlChange: _onUrlChange,
}: IUploadFileProps) => {
  const [files, setFiles] = useState<Array<File & { preview: string }>>([]);
  const [error, setError] = useState<string | null>(null);

  const { getRootProps, getInputProps, isFocused, isDragAccept, isDragReject } =
    useDropzone({
      onDrop: (acceptedFiles, fileRejections) => {
        setError(null);
        setFiles(
          acceptedFiles.map((file) =>
            Object.assign(file, {
              preview: URL.createObjectURL(file),
            }),
          ),
        );
        fileRejections.forEach(({ errors }) => {
          errors.forEach(({ message }) => {
            setError(message);
          });
        });
      },
      maxFiles: 1,
      maxSize: 20 * 1024 * 1024, // 20 MB
      multiple: false,
    });

  useEffect(() => {
    return () => files.forEach((file) => URL.revokeObjectURL(file.preview));
  }, [files]);

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
    <div style={thumb} key={file.name}>
      <div style={thumbInner}>
        {file.type.startsWith('image/') ? (
          <img src={file.preview} style={img} alt={file.name} />
        ) : file.type.startsWith('video/') ? (
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
          {file.name}
        </div>
      </div>
    </div>
  ));

  const handleUploadFile = async () => {
    if (files.length === 0) {
      setError('Please upload a file');
      return;
    }
  };

  return (
    <div className="container small">
      <div className="fn_cs_contact_form">
        <ul>
          <li>
            <section>
              <div {...getRootProps({ style })}>
                <input {...getInputProps()} />
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
            disabled={files.length === 0}
            style={{ border: 'none', marginBottom: 24, zIndex: 1 }}
            onClick={handleUploadFile}
          >
            <span>Upload file</span>
          </button>
        )}

        <ul style={{ marginTop: '200px' }}>
          <li>
            <button
              className="metaportal_fn_button full cursor"
              style={{
                border: 'none',
                zIndex: 1,
              }}
              onClick={onBack}
            >
              <span>Back</span>
            </button>
          </li>
          <li>
            <button
              className="metaportal_fn_button full cursor"
              disabled={files.length === 0}
              style={{
                border: 'none',
                zIndex: 1,
                marginBottom: 24,
              }}
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
