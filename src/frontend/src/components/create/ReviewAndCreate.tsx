import { Box, CircularProgress } from '@mui/material';
import moment from 'moment';
import { useCrnlToken } from '../../hooks/useCrnlToken';
import { useChronolock } from '../../hooks/useChronolock';
import {
  DerivedPublicKey,
  IbeCiphertext,
  IbeIdentity,
  IbeSeed,
} from '@dfinity/vetkeys';
import { Principal } from '@dfinity/principal';

interface IReviewAndCreate {
  name: string | undefined;
  title: string | undefined;
  description: string | undefined;
  attributes: { key: string; value: string }[] | undefined;
  mediaUrl: string | undefined;
  mediaId: string | undefined;
  fileType: string | undefined;
  lockTime: number | null | undefined;
  recipients: string[] | undefined;
  cryptoKey: CryptoKey | null | undefined;
  onBack: () => void;
}
const ReviewAndCreate = ({
  name,
  title,
  description,
  attributes,
  mediaUrl,
  mediaId,
  fileType,
  lockTime,
  recipients,
  cryptoKey,
  onBack,
}: IReviewAndCreate) => {
  const date = lockTime ? moment(lockTime * 1000) : null;
  const { balanceData } = useCrnlToken();
  const {
    getVetkdPublicKey,
    createChronolock,
    createChronolockLoading,
    isGetVetkdPublicKeyLoading,
  } = useChronolock();

  const notEnoughCrnl = parseFloat(balanceData) < 20;
  const showCreditError = notEnoughCrnl && mediaUrl && mediaUrl;
  const isLoading = createChronolockLoading || isGetVetkdPublicKeyLoading;

  const handleCreate = async () => {
    if (cryptoKey) {
      const iv = window.crypto.getRandomValues(new Uint8Array(12));
      const secureMetaData = {
        name,
        description,
        fileType,
        mediaUrl,
        mediaId,
        attributes,
      };
      const encodedMetaData = new TextEncoder().encode(
        JSON.stringify(secureMetaData),
      );

      const encryptedBuffer = await window.crypto.subtle.encrypt(
        { name: 'AES-GCM', iv },
        cryptoKey,
        encodedMetaData,
      );
      const concatenatedArray = new Uint8Array(
        iv.length + encryptedBuffer.byteLength,
      );
      concatenatedArray.set(iv, 0);
      concatenatedArray.set(new Uint8Array(encryptedBuffer), iv.length);
      const concatenatedBuffer = concatenatedArray.buffer;
      const encryptedBase64 = btoa(
        String.fromCharCode(...new Uint8Array(concatenatedBuffer)),
      );
      const unsecretMetaData: {
        title: string | undefined;
        lockTime: number | null | undefined;
        userKeys: { user: string; key: string }[];
        encryptedMetaData: string;
      } = {
        title,
        lockTime,
        userKeys: [],
        encryptedMetaData: encryptedBase64,
      };

      const rawKey = await window.crypto.subtle.exportKey('raw', cryptoKey);
      const rawKeyUint8 = new Uint8Array(rawKey);
      const vetkdPublicKeyObject = await getVetkdPublicKey();
      const vetkdPublicKeyBuffer = (
        vetkdPublicKeyObject as { Ok: { public_key: ArrayBuffer } }
      ).Ok.public_key;

      const vetkdPublicKey = DerivedPublicKey.deserialize(
        new Uint8Array(vetkdPublicKeyBuffer),
      );

      if (recipients && recipients.length > 0 && vetkdPublicKey) {
        recipients?.map((recipient) => {
          if (recipient) {
            const encryptedKey = IbeCiphertext.encrypt(
              vetkdPublicKey,
              IbeIdentity.fromString(`${recipient}:${lockTime}`),
              rawKeyUint8,
              IbeSeed.random(),
            );
            const encryptedKeyBase64 = btoa(
              String.fromCharCode(...new Uint8Array(encryptedKey.serialize())),
            );
            unsecretMetaData.userKeys.push({
              user: recipient,
              key: encryptedKeyBase64,
            });
          }
        });
      } else {
        const encryptedKey = IbeCiphertext.encrypt(
          vetkdPublicKey,
          IbeIdentity.fromString(lockTime?.toString() || ''),
          rawKeyUint8,
          IbeSeed.random(),
        );
        const encryptedKeyBase64 = btoa(
          String.fromCharCode(...new Uint8Array(encryptedKey.serialize())),
        );
        unsecretMetaData.userKeys.push({
          user: 'public',
          key: encryptedKeyBase64,
        });
      }

      const unsecureMetaDataBase64 = btoa(JSON.stringify(unsecretMetaData));
      setTimeout(async () => {
        const chronolock = await createChronolock([unsecureMetaDataBase64]);
        console.log(
          'Creating Chronolock',
          unsecretMetaData,
          unsecureMetaDataBase64,
          chronolock,
        );
      }, 0);
    } else {
      console.error('Crypto key is not defined');
      return;
    }
  };

  return (
    <div className="container small">
      <div className="fn_cs_contact_form">
        <Box
          mb={3}
          textAlign="center"
          sx={{ border: '1px solid #ccc', padding: '32px' }}
        >
          <h2>Review and Create</h2>
        </Box>
        <Box mb={4} gap={2} display={'flex'} flexDirection="column">
          <h3>
            Please review bellow details carefully. If everything looks good,
            click "Create" to proceed.
          </h3>
          <Box
            sx={{
              backgroundColor: '#f0f0f0',
              padding: '24px',
              borderRadius: '8px',
            }}
          >
            <h5 style={{ color: 'green' }}>
              Note: All metadata and files are securely encrypted using AES-GCM,
              with the encryption key further protected using ICP IBE Vetkey
              Encryption. Only the specified recipients will be able to access
              the Chronolock after the unlock time. If no recipients are set,
              the Chronolock will be accessible to everyone after the unlock
              time. Please ensure the recipients' principals are accurate. It
              will cost you 20 $CRNL to create a Media Chronolock. Text only
              Chronolocks can be created for free. Make sure to have enough
              $CRNL in your wallet to cover the creation cost.
            </h5>
          </Box>
        </Box>
        <ul>
          <li>
            <h2>Name</h2>
            <p>{name}</p>
          </li>
          <li>
            <h2>Title</h2>
            <p>{title}</p>
          </li>
          <li>
            <h2>Description</h2>
            <p>{description}</p>
          </li>

          <li>
            <h2>Media URL</h2>
            <p>{mediaUrl}</p>
          </li>
          <li>
            <h2>Media ID</h2>
            <p>{mediaId}</p>
          </li>
          <li>
            <h2>Lock Time</h2>
            <p>{date ? date.toLocaleString() : 'Not set'}</p>
          </li>
          <li>
            <h2>File Type</h2>
            <p>{fileType}</p>
          </li>
          <li>
            <h2>Recipients</h2>
            <ul style={{ padding: '24px 0' }}>
              {recipients?.length ? (
                recipients.map((recipient, index) => (
                  <li key={index}>{recipient}</li>
                ))
              ) : (
                <li>No recipients set</li>
              )}
            </ul>
          </li>
          <li>
            <h2>Attributes</h2>
            <ul style={{ padding: '24px 0' }}>
              {attributes && attributes.length > 0 ? (
                attributes.map((attr, index) => (
                  <li key={index}>
                    {attr.key}: {attr.value}
                  </li>
                ))
              ) : (
                <li>No attributes set</li>
              )}
            </ul>
          </li>
        </ul>
        {showCreditError && (
          <Box
            sx={{
              backgroundColor: '#f0f0f0',
              padding: '24px',
              borderRadius: '8px',
            }}
            mt={2}
          >
            <h5 style={{ color: 'red' }}>
              You don't have enough $CRNL to create a Media Chronolock.
            </h5>
          </Box>
        )}
        <ul style={{ marginTop: '100px' }}>
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
              style={{
                border: 'none',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                cursor: isLoading ? 'not-allowed' : 'pointer',
              }}
              disabled={
                !name ||
                !title ||
                !description ||
                !lockTime ||
                !cryptoKey ||
                isLoading ||
                notEnoughCrnl
              }
              onClick={handleCreate}
            >
              <Box
                mx={2}
                display="flex"
                alignItems="center"
                justifyContent="center"
                width={100}
                position="relative"
              >
                <span>Create</span>
                {isLoading && (
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
          </li>
        </ul>
      </div>
    </div>
  );
};

export default ReviewAndCreate;
