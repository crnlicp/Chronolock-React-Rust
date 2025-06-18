import { useCallback } from 'react';
import { useActor } from '../ActorContextProvider';

interface IUseChronolock {
  isUploadLoading: boolean;
  uploadErrors: (Error | undefined)[];
  isGetMediaLoading: boolean;
  isGetVetkdPublicKeyLoading: boolean;
  getMediaError?: Error;
  createChronolockLoading?: boolean;
  createChronolockError?: Error;
  createChronolock: (
    eventOrReplaceArgs?:
      | unknown[]
      | React.MouseEvent<Element, MouseEvent>
      | undefined,
  ) => Promise<unknown>;
  upload: (media: ArrayBuffer) => Promise<unknown>;
  getMediaChunked: (
    mediaId: string,
    totalSize: number,
  ) => Promise<Uint8Array<ArrayBuffer>>;
  getVetkdPublicKey: () => Promise<unknown>;
  generateKey: () => Promise<CryptoKey>;
}

const UPLOAD_CHUNK_SIZE = 1.95 * 1024 * 1024; // 2MB
const DOWNLOAD_CHUNK_SIZE = 2.5 * 1024 * 1024; // 2.5MB

export const useChronolock = (): IUseChronolock => {
  const {
    chronolockActor: {
      useQueryCall: chronolockQueryCall,
      useUpdateCall: chronolockUpdateCall,
    },
  } = useActor();

  const { call: getVetkdPublicKey, loading: isGetVetkdPublicKeyLoading } =
    chronolockQueryCall({
      refetchOnMount: false,
      functionName: 'ibe_encryption_key' as any,
    });

  const {
    call: getMediaChunk,
    loading: isGetMediaLoading,
    error: getMediaError,
  } = chronolockQueryCall({
    refetchOnMount: false,
    functionName: 'get_media_chunk' as any,
  });

  const {
    call: startMediaUpload,
    loading: startMediaUploadLoading,
    error: startMediaUploadError,
  } = chronolockUpdateCall({
    functionName: 'start_media_upload' as any,
  });

  const {
    call: uploadMediaChunk,
    loading: uploadMediaChunkLoading,
    error: uploadMediaChunkError,
  } = chronolockUpdateCall({
    functionName: 'upload_media_chunk' as any,
  });

  const {
    call: finishMediaUpload,
    loading: finishMediaUploadLoading,
    error: finishMediaUploadError,
  } = chronolockUpdateCall({
    functionName: 'finish_media_upload' as any,
  });

  const {
    call: createChronolock,
    loading: createChronolockLoading,
    error: createChronolockError,
  } = chronolockUpdateCall({
    functionName: 'create_chronolock' as any,
  });

  const upload = useCallback(
    async (media: ArrayBuffer) => {
      const totalChunks = Math.ceil(media.byteLength / UPLOAD_CHUNK_SIZE);
      // 1. Start upload, get media_id
      const mediaId = await startMediaUpload([totalChunks]);
      console.log('mediaId:', mediaId);
      // 2. Upload each chunk
      for (let i = 0; i < totalChunks; i++) {
        const start = i * UPLOAD_CHUNK_SIZE;
        const end = Math.min(start + UPLOAD_CHUNK_SIZE, media.byteLength);
        const chunk = new Uint8Array(media.slice(start, end));
        const upload = await uploadMediaChunk([mediaId, i, Array.from(chunk)]);
        console.log('upload chunk:', upload);
      }
      // 3. Finish upload, get URL
      const urlObject = await finishMediaUpload([mediaId]);
      return { urlObject, mediaId };
    },
    [startMediaUpload, uploadMediaChunk, finishMediaUpload],
  );

  const getMediaChunked = useCallback(
    async (mediaId: string, totalSize: number) => {
      let chunks: Uint8Array[] = [];
      const chunkSize = DOWNLOAD_CHUNK_SIZE;

      for (let offset = 0; offset < totalSize; offset += chunkSize) {
        const length = Math.min(chunkSize, totalSize - offset);
        const chunkResponse = await getMediaChunk([mediaId, offset, length]);

        // Extract the actual chunk from the response
        const chunk = (chunkResponse as { Ok?: Uint8Array | number[] | object })
          ?.Ok;

        let uint8Chunk: Uint8Array | undefined = undefined;
        if (chunk instanceof Uint8Array) {
          uint8Chunk = chunk;
        } else if (Array.isArray(chunk)) {
          uint8Chunk = Uint8Array.from(chunk);
        } else if (chunk && typeof chunk === 'object') {
          const values = Object.values(chunk);
          uint8Chunk = Uint8Array.from(values as number[]);
        } else {
          console.warn('Unexpected chunk type:', chunk);
        }

        if (!uint8Chunk || uint8Chunk.length === 0) {
          console.warn('Empty or invalid chunk received, breaking the loop.');
          break;
        }

        console.log('Chunk received:', uint8Chunk.length, { Ok: uint8Chunk });

        chunks.push(uint8Chunk);
      }

      const totalLength = chunks.reduce((acc, cur) => acc + cur.length, 0);
      const result = new Uint8Array(totalLength);
      let pos = 0;
      for (const chunk of chunks) {
        result.set(chunk, pos);
        pos += chunk.length;
      }
      return result;
    },
    [getMediaChunk],
  );

  const generateKey = async () => {
    const generatedKey = await window.crypto.subtle.generateKey(
      {
        name: 'AES-GCM',
        length: 256,
      },
      true,
      ['encrypt', 'decrypt'],
    );
    console.log('Crypto Key Generated!');
    return generatedKey;
  };

  const isUploadLoading =
    startMediaUploadLoading ||
    uploadMediaChunkLoading ||
    finishMediaUploadLoading ||
    isGetVetkdPublicKeyLoading;

  const uploadErrors = [
    startMediaUploadError,
    uploadMediaChunkError,
    finishMediaUploadError,
  ].filter((error) => error !== undefined);

  return {
    isUploadLoading,
    uploadErrors,
    isGetMediaLoading,
    isGetVetkdPublicKeyLoading,
    getMediaError,
    createChronolockLoading,
    createChronolockError,
    createChronolock,
    upload,
    getMediaChunked,
    generateKey,
    getVetkdPublicKey,
  };
};
