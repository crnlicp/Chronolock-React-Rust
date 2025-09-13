import { useCallback } from 'react';
import { useActor } from '../ActorContextProvider';
import { Principal } from '@dfinity/principal';

export interface Chronolock {
  id: string;
  owner: string;
  metadata: string;
}

interface IUseChronolock {
  isUploadLoading: boolean;
  uploadErrors: (Error | undefined)[];
  isGetMediaLoading: boolean;
  isGetVetkdPublicKeyLoading: boolean;
  getMediaError?: Error;
  isCreateChronolockLoading?: boolean;
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
  // Decryption functions
  getTimeDecryptionKey: (
    unlockTimeHex: string,
    transportPublicKey: number[],
  ) => Promise<unknown>;
  getUserTimeDecryptionKey: (
    unlockTimeHex: string,
    userId: string,
    transportPublicKey: number[],
  ) => Promise<unknown>;
  // New pagination functions
  getAllChronolocksCount: () => Promise<unknown>;
  getOwnerChronolocksCount: (owner: string) => Promise<unknown>;
  getUserAccessibleChronolocksCount: (user: string) => Promise<unknown>;
  getAllChronolocksPaginated: (
    offset: number,
    limit: number,
  ) => Promise<unknown>;
  getOwnerChronolocksPaginated: (
    owner: string,
    offset: number,
    limit: number,
  ) => Promise<unknown>;
  getUserAccessibleChronolocksPaginated: (
    user: string,
    offset: number,
    limit: number,
  ) => Promise<unknown>;
  // Loading states for new functions
  isGetTimeDecryptionKeyLoading: boolean;
  isGetUserTimeDecryptionKeyLoading: boolean;
  isGetAllChronolocksLoading: boolean;
  isGetOwnerChronolocksLoading: boolean;
  isGetUserAccessibleChronolocksLoading: boolean;
  isGetAllChronolocksCountLoading: boolean;
  isGetOwnerChronolocksCountLoading: boolean;
  isGetUserAccessibleChronolocksCountLoading: boolean;
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

  const { call: getTimeDecryptionKey, loading: isGetTimeDecryptionKeyLoading } =
    chronolockUpdateCall({
      functionName: 'get_time_decryption_key' as any,
    });

  const {
    call: getUserTimeDecryptionKey,
    loading: isGetUserTimeDecryptionKeyLoading,
  } = chronolockUpdateCall({
    functionName: 'get_user_time_decryption_key' as any,
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
    loading: isCreateChronolockLoading,
    error: createChronolockError,
  } = chronolockUpdateCall({
    functionName: 'create_chronolock' as any,
  });

  const {
    call: getAllChronolocksCount,
    loading: isGetAllChronolocksCountLoading,
  } = chronolockQueryCall({
    refetchOnMount: false,
    functionName: 'get_total_chronolocks_count' as any,
  });

  const {
    call: getOwnerChronolocksCountCall,
    loading: isGetOwnerChronolocksCountLoading,
  } = chronolockQueryCall({
    refetchOnMount: false,
    functionName: 'get_owner_chronolocks_count' as any,
  });

  const {
    call: getUserAccessibleChronolocksCountCall,
    loading: isGetUserAccessibleChronolocksCountLoading,
  } = chronolockQueryCall({
    refetchOnMount: false,
    functionName: 'get_user_accessible_chronolocks_count' as any,
  });

  const {
    call: getAllChronolocksPaginatedCall,
    loading: isGetAllChronolocksLoading,
  } = chronolockQueryCall({
    refetchOnMount: false,
    functionName: 'get_all_chronolocks_paginated' as any,
  });

  const {
    call: getOwnerChronolocksPaginatedCall,
    loading: isGetOwnerChronolocksLoading,
  } = chronolockQueryCall({
    refetchOnMount: false,
    functionName: 'get_owner_chronolocks_paginated' as any,
  });

  const {
    call: getUserAccessibleChronolocksPaginatedCall,
    loading: isGetUserAccessibleChronolocksLoading,
  } = chronolockQueryCall({
    refetchOnMount: false,
    functionName: 'get_user_accessible_chronolocks_paginated' as any,
  });

  const getOwnerChronolocksCount = useCallback(
    (owner: string) => {
      const principalOwner = Principal.fromText(owner);
      return getOwnerChronolocksCountCall([principalOwner]);
    },
    [getOwnerChronolocksCountCall],
  );

  const getUserAccessibleChronolocksCount = useCallback(
    (user: string) => {
      const principalUser = Principal.fromText(user);
      return getUserAccessibleChronolocksCountCall([principalUser]);
    },
    [getUserAccessibleChronolocksCountCall],
  );

  const getAllChronolocksPaginated = useCallback(
    (offset: number, limit: number) =>
      getAllChronolocksPaginatedCall([offset, limit]),
    [getAllChronolocksPaginatedCall],
  );

  const getOwnerChronolocksPaginated = useCallback(
    (owner: string, offset: number, limit: number) => {
      const principalOwner = Principal.fromText(owner);
      return getOwnerChronolocksPaginatedCall([principalOwner, offset, limit]);
    },
    [getOwnerChronolocksPaginatedCall],
  );

  const getUserAccessibleChronolocksPaginated = useCallback(
    (user: string, offset: number, limit: number) => {
      const principalUser = Principal.fromText(user);
      return getUserAccessibleChronolocksPaginatedCall([
        principalUser,
        offset,
        limit,
      ]);
    },
    [getUserAccessibleChronolocksPaginatedCall],
  );

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

  const generateKey = useCallback(async () => {
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
  }, []);

  const getTimeDecryptionKeyWrapped = useCallback(
    (unlockTimeHex: string, transportPublicKey: number[]) => {
      return getTimeDecryptionKey([unlockTimeHex, transportPublicKey]);
    },
    [getTimeDecryptionKey],
  );

  const getUserTimeDecryptionKeyWrapped = useCallback(
    (unlockTimeHex: string, userId: string, transportPublicKey: number[]) => {
      return getUserTimeDecryptionKey([
        unlockTimeHex,
        userId,
        transportPublicKey,
      ]);
    },
    [getUserTimeDecryptionKey],
  );

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
    isCreateChronolockLoading,
    createChronolockError,
    createChronolock,
    upload,
    getMediaChunked,
    generateKey,
    getVetkdPublicKey,
    // Decryption functions
    getTimeDecryptionKey: getTimeDecryptionKeyWrapped,
    getUserTimeDecryptionKey: getUserTimeDecryptionKeyWrapped,
    // New pagination functions
    getAllChronolocksCount,
    getOwnerChronolocksCount,
    getUserAccessibleChronolocksCount,
    getAllChronolocksPaginated,
    getOwnerChronolocksPaginated,
    getUserAccessibleChronolocksPaginated,
    // Loading states for new functions
    isGetTimeDecryptionKeyLoading,
    isGetUserTimeDecryptionKeyLoading,
    isGetAllChronolocksLoading,
    isGetOwnerChronolocksLoading,
    isGetUserAccessibleChronolocksLoading,
    isGetAllChronolocksCountLoading,
    isGetOwnerChronolocksCountLoading,
    isGetUserAccessibleChronolocksCountLoading,
  };
};
