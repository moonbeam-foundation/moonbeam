pragma solidity ^0.8.0;

contract StorageFiller {
    // Mapping to store large byte arrays
    mapping(uint256 => bytes) public largeStorage;

    // Fill a single storage slot with a large value
    function fillStorage(uint256 slot, uint256 size) public {
        bytes memory data = new bytes(size);
        // Fill with non-zero data to ensure it's stored
        for (uint i = 0; i < size; i++) {
            data[i] = bytes1(uint8((slot + i) % 256));
        }
        largeStorage[slot] = data;
    }

    // Fill multiple storage slots in one transaction
    function fillStorageBatch(
        uint256 startSlot,
        uint256 count,
        uint256 size
    ) public {
        for (uint256 i = 0; i < count; i++) {
            fillStorage(startSlot + i, size);
        }
    }

    // Modify all existing slots with a small value
    function modifyStorage(uint256 slot) public returns (bytes memory) {
        for (uint256 i = 0; i < slot + 1; i++) {
            if (largeStorage[i].length > 0) {
                bytes memory data = largeStorage[i];
                // Modify each byte in the existing data
                for (uint256 j = 0; j < data.length; j++) {
                    data[j] = bytes1(uint8((i + j) % 256));
                }
                largeStorage[i] = data;
            }
        }
        return largeStorage[slot];
    }

    // Modify multiple storage slots in one transaction
    function modifyStorageBatch(
        uint256 startSlot,
        uint256 count
    ) external returns (uint256) {
        uint256 totalSize = 0;
        for (uint256 i = 0; i < count; i++) {
            totalSize += modifyStorage(startSlot + i).length;
        }
        return totalSize;
    }

    // Read a single storage slot
    function readStorage(uint256 slot) public view returns (bytes memory) {
        return largeStorage[slot];
    }

    // Read multiple storage slots in one transaction
    function readStorageBatch(
        uint256 startSlot,
        uint256 count
    ) public view returns (uint256) {
        uint256 totalSize = 0;
        for (uint256 i = 0; i < count; i++) {
            totalSize += readStorage(startSlot + i).length;
        }
        return totalSize;
    }

    // Read exactly 256 storage slots without loops
    function read384Slots(uint256 slot) public view returns (uint256) {
        uint256 totalSize = 0;

        // Slots 0-63
        totalSize += largeStorage[slot].length;
        totalSize += largeStorage[slot + 1].length;
        totalSize += largeStorage[slot + 2].length;
        totalSize += largeStorage[slot + 3].length;
        totalSize += largeStorage[slot + 4].length;
        totalSize += largeStorage[slot + 5].length;
        totalSize += largeStorage[slot + 6].length;
        totalSize += largeStorage[slot + 7].length;
        totalSize += largeStorage[slot + 8].length;
        totalSize += largeStorage[slot + 9].length;
        totalSize += largeStorage[slot + 10].length;
        totalSize += largeStorage[slot + 11].length;
        totalSize += largeStorage[slot + 12].length;
        totalSize += largeStorage[slot + 13].length;
        totalSize += largeStorage[slot + 14].length;
        totalSize += largeStorage[slot + 15].length;
        totalSize += largeStorage[slot + 16].length;
        totalSize += largeStorage[slot + 17].length;
        totalSize += largeStorage[slot + 18].length;
        totalSize += largeStorage[slot + 19].length;
        totalSize += largeStorage[slot + 20].length;
        totalSize += largeStorage[slot + 21].length;
        totalSize += largeStorage[slot + 22].length;
        totalSize += largeStorage[slot + 23].length;
        totalSize += largeStorage[slot + 24].length;
        totalSize += largeStorage[slot + 25].length;
        totalSize += largeStorage[slot + 26].length;
        totalSize += largeStorage[slot + 27].length;
        totalSize += largeStorage[slot + 28].length;
        totalSize += largeStorage[slot + 29].length;
        totalSize += largeStorage[slot + 30].length;
        totalSize += largeStorage[slot + 31].length;
        totalSize += largeStorage[slot + 32].length;
        totalSize += largeStorage[slot + 33].length;
        totalSize += largeStorage[slot + 34].length;
        totalSize += largeStorage[slot + 35].length;
        totalSize += largeStorage[slot + 36].length;
        totalSize += largeStorage[slot + 37].length;
        totalSize += largeStorage[slot + 38].length;
        totalSize += largeStorage[slot + 39].length;
        totalSize += largeStorage[slot + 40].length;
        totalSize += largeStorage[slot + 41].length;
        totalSize += largeStorage[slot + 42].length;
        totalSize += largeStorage[slot + 43].length;
        totalSize += largeStorage[slot + 44].length;
        totalSize += largeStorage[slot + 45].length;
        totalSize += largeStorage[slot + 46].length;
        totalSize += largeStorage[slot + 47].length;
        totalSize += largeStorage[slot + 48].length;
        totalSize += largeStorage[slot + 49].length;
        totalSize += largeStorage[slot + 50].length;
        totalSize += largeStorage[slot + 51].length;
        totalSize += largeStorage[slot + 52].length;
        totalSize += largeStorage[slot + 53].length;
        totalSize += largeStorage[slot + 54].length;
        totalSize += largeStorage[slot + 55].length;
        totalSize += largeStorage[slot + 56].length;
        totalSize += largeStorage[slot + 57].length;
        totalSize += largeStorage[slot + 58].length;
        totalSize += largeStorage[slot + 59].length;
        totalSize += largeStorage[slot + 60].length;
        totalSize += largeStorage[slot + 61].length;
        totalSize += largeStorage[slot + 62].length;
        totalSize += largeStorage[slot + 63].length;

        // Slots 64-127
        totalSize += largeStorage[slot + 64].length;
        totalSize += largeStorage[slot + 65].length;
        totalSize += largeStorage[slot + 66].length;
        totalSize += largeStorage[slot + 67].length;
        totalSize += largeStorage[slot + 68].length;
        totalSize += largeStorage[slot + 69].length;
        totalSize += largeStorage[slot + 70].length;
        totalSize += largeStorage[slot + 71].length;
        totalSize += largeStorage[slot + 72].length;
        totalSize += largeStorage[slot + 73].length;
        totalSize += largeStorage[slot + 74].length;
        totalSize += largeStorage[slot + 75].length;
        totalSize += largeStorage[slot + 76].length;
        totalSize += largeStorage[slot + 77].length;
        totalSize += largeStorage[slot + 78].length;
        totalSize += largeStorage[slot + 79].length;
        totalSize += largeStorage[slot + 80].length;
        totalSize += largeStorage[slot + 81].length;
        totalSize += largeStorage[slot + 82].length;
        totalSize += largeStorage[slot + 83].length;
        totalSize += largeStorage[slot + 84].length;
        totalSize += largeStorage[slot + 85].length;
        totalSize += largeStorage[slot + 86].length;
        totalSize += largeStorage[slot + 87].length;
        totalSize += largeStorage[slot + 88].length;
        totalSize += largeStorage[slot + 89].length;
        totalSize += largeStorage[slot + 90].length;
        totalSize += largeStorage[slot + 91].length;
        totalSize += largeStorage[slot + 92].length;
        totalSize += largeStorage[slot + 93].length;
        totalSize += largeStorage[slot + 94].length;
        totalSize += largeStorage[slot + 95].length;
        totalSize += largeStorage[slot + 96].length;
        totalSize += largeStorage[slot + 97].length;
        totalSize += largeStorage[slot + 98].length;
        totalSize += largeStorage[slot + 99].length;
        totalSize += largeStorage[slot + 100].length;
        totalSize += largeStorage[slot + 101].length;
        totalSize += largeStorage[slot + 102].length;
        totalSize += largeStorage[slot + 103].length;
        totalSize += largeStorage[slot + 104].length;
        totalSize += largeStorage[slot + 105].length;
        totalSize += largeStorage[slot + 106].length;
        totalSize += largeStorage[slot + 107].length;
        totalSize += largeStorage[slot + 108].length;
        totalSize += largeStorage[slot + 109].length;
        totalSize += largeStorage[slot + 110].length;
        totalSize += largeStorage[slot + 111].length;
        totalSize += largeStorage[slot + 112].length;
        totalSize += largeStorage[slot + 113].length;
        totalSize += largeStorage[slot + 114].length;
        totalSize += largeStorage[slot + 115].length;
        totalSize += largeStorage[slot + 116].length;
        totalSize += largeStorage[slot + 117].length;
        totalSize += largeStorage[slot + 118].length;
        totalSize += largeStorage[slot + 119].length;
        totalSize += largeStorage[slot + 120].length;
        totalSize += largeStorage[slot + 121].length;
        totalSize += largeStorage[slot + 122].length;
        totalSize += largeStorage[slot + 123].length;
        totalSize += largeStorage[slot + 124].length;
        totalSize += largeStorage[slot + 125].length;
        totalSize += largeStorage[slot + 126].length;
        totalSize += largeStorage[slot + 127].length;

        // Slots 128-191
        totalSize += largeStorage[slot + 128].length;
        totalSize += largeStorage[slot + 129].length;
        totalSize += largeStorage[slot + 130].length;
        totalSize += largeStorage[slot + 131].length;
        totalSize += largeStorage[slot + 132].length;
        totalSize += largeStorage[slot + 133].length;
        totalSize += largeStorage[slot + 134].length;
        totalSize += largeStorage[slot + 135].length;
        totalSize += largeStorage[slot + 136].length;
        totalSize += largeStorage[slot + 137].length;
        totalSize += largeStorage[slot + 138].length;
        totalSize += largeStorage[slot + 139].length;
        totalSize += largeStorage[slot + 140].length;
        totalSize += largeStorage[slot + 141].length;
        totalSize += largeStorage[slot + 142].length;
        totalSize += largeStorage[slot + 143].length;
        totalSize += largeStorage[slot + 144].length;
        totalSize += largeStorage[slot + 145].length;
        totalSize += largeStorage[slot + 146].length;
        totalSize += largeStorage[slot + 147].length;
        totalSize += largeStorage[slot + 148].length;
        totalSize += largeStorage[slot + 149].length;
        totalSize += largeStorage[slot + 150].length;
        totalSize += largeStorage[slot + 151].length;
        totalSize += largeStorage[slot + 152].length;
        totalSize += largeStorage[slot + 153].length;
        totalSize += largeStorage[slot + 154].length;
        totalSize += largeStorage[slot + 155].length;
        totalSize += largeStorage[slot + 156].length;
        totalSize += largeStorage[slot + 157].length;
        totalSize += largeStorage[slot + 158].length;
        totalSize += largeStorage[slot + 159].length;
        totalSize += largeStorage[slot + 160].length;
        totalSize += largeStorage[slot + 161].length;
        totalSize += largeStorage[slot + 162].length;
        totalSize += largeStorage[slot + 163].length;
        totalSize += largeStorage[slot + 164].length;
        totalSize += largeStorage[slot + 165].length;
        totalSize += largeStorage[slot + 166].length;
        totalSize += largeStorage[slot + 167].length;
        totalSize += largeStorage[slot + 168].length;
        totalSize += largeStorage[slot + 169].length;
        totalSize += largeStorage[slot + 170].length;
        totalSize += largeStorage[slot + 171].length;
        totalSize += largeStorage[slot + 172].length;
        totalSize += largeStorage[slot + 173].length;
        totalSize += largeStorage[slot + 174].length;
        totalSize += largeStorage[slot + 175].length;
        totalSize += largeStorage[slot + 176].length;
        totalSize += largeStorage[slot + 177].length;
        totalSize += largeStorage[slot + 178].length;
        totalSize += largeStorage[slot + 179].length;
        totalSize += largeStorage[slot + 180].length;
        totalSize += largeStorage[slot + 181].length;
        totalSize += largeStorage[slot + 182].length;
        totalSize += largeStorage[slot + 183].length;
        totalSize += largeStorage[slot + 184].length;
        totalSize += largeStorage[slot + 185].length;
        totalSize += largeStorage[slot + 186].length;
        totalSize += largeStorage[slot + 187].length;
        totalSize += largeStorage[slot + 188].length;
        totalSize += largeStorage[slot + 189].length;
        totalSize += largeStorage[slot + 190].length;
        totalSize += largeStorage[slot + 191].length;

        // Slots 192-255
        totalSize += largeStorage[slot + 192].length;
        totalSize += largeStorage[slot + 193].length;
        totalSize += largeStorage[slot + 194].length;
        totalSize += largeStorage[slot + 195].length;
        totalSize += largeStorage[slot + 196].length;
        totalSize += largeStorage[slot + 197].length;
        totalSize += largeStorage[slot + 198].length;
        totalSize += largeStorage[slot + 199].length;
        totalSize += largeStorage[slot + 200].length;
        totalSize += largeStorage[slot + 201].length;
        totalSize += largeStorage[slot + 202].length;
        totalSize += largeStorage[slot + 203].length;
        totalSize += largeStorage[slot + 204].length;
        totalSize += largeStorage[slot + 205].length;
        totalSize += largeStorage[slot + 206].length;
        totalSize += largeStorage[slot + 207].length;
        totalSize += largeStorage[slot + 208].length;
        totalSize += largeStorage[slot + 209].length;
        totalSize += largeStorage[slot + 210].length;
        totalSize += largeStorage[slot + 211].length;
        totalSize += largeStorage[slot + 212].length;
        totalSize += largeStorage[slot + 213].length;
        totalSize += largeStorage[slot + 214].length;
        totalSize += largeStorage[slot + 215].length;
        totalSize += largeStorage[slot + 216].length;
        totalSize += largeStorage[slot + 217].length;
        totalSize += largeStorage[slot + 218].length;
        totalSize += largeStorage[slot + 219].length;
        totalSize += largeStorage[slot + 220].length;
        totalSize += largeStorage[slot + 221].length;
        totalSize += largeStorage[slot + 222].length;
        totalSize += largeStorage[slot + 223].length;
        totalSize += largeStorage[slot + 224].length;
        totalSize += largeStorage[slot + 225].length;
        totalSize += largeStorage[slot + 226].length;
        totalSize += largeStorage[slot + 227].length;
        totalSize += largeStorage[slot + 228].length;
        totalSize += largeStorage[slot + 229].length;
        totalSize += largeStorage[slot + 230].length;
        totalSize += largeStorage[slot + 231].length;
        totalSize += largeStorage[slot + 232].length;
        totalSize += largeStorage[slot + 233].length;
        totalSize += largeStorage[slot + 234].length;
        totalSize += largeStorage[slot + 235].length;
        totalSize += largeStorage[slot + 236].length;
        totalSize += largeStorage[slot + 237].length;
        totalSize += largeStorage[slot + 238].length;
        totalSize += largeStorage[slot + 239].length;
        totalSize += largeStorage[slot + 240].length;
        totalSize += largeStorage[slot + 241].length;
        totalSize += largeStorage[slot + 242].length;
        totalSize += largeStorage[slot + 243].length;
        totalSize += largeStorage[slot + 244].length;
        totalSize += largeStorage[slot + 245].length;
        totalSize += largeStorage[slot + 246].length;
        totalSize += largeStorage[slot + 247].length;
        totalSize += largeStorage[slot + 248].length;
        totalSize += largeStorage[slot + 249].length;
        totalSize += largeStorage[slot + 250].length;
        totalSize += largeStorage[slot + 251].length;
        totalSize += largeStorage[slot + 252].length;
        totalSize += largeStorage[slot + 253].length;
        totalSize += largeStorage[slot + 254].length;
        totalSize += largeStorage[slot + 255].length;

        // Slots 256-319
        totalSize += largeStorage[slot + 256].length;
        totalSize += largeStorage[slot + 257].length;
        totalSize += largeStorage[slot + 258].length;
        totalSize += largeStorage[slot + 259].length;
        totalSize += largeStorage[slot + 260].length;
        totalSize += largeStorage[slot + 261].length;
        totalSize += largeStorage[slot + 262].length;
        totalSize += largeStorage[slot + 263].length;
        totalSize += largeStorage[slot + 264].length;
        totalSize += largeStorage[slot + 265].length;
        totalSize += largeStorage[slot + 266].length;
        totalSize += largeStorage[slot + 267].length;
        totalSize += largeStorage[slot + 268].length;
        totalSize += largeStorage[slot + 269].length;
        totalSize += largeStorage[slot + 270].length;
        totalSize += largeStorage[slot + 271].length;
        totalSize += largeStorage[slot + 272].length;
        totalSize += largeStorage[slot + 273].length;
        totalSize += largeStorage[slot + 274].length;
        totalSize += largeStorage[slot + 275].length;
        totalSize += largeStorage[slot + 276].length;
        totalSize += largeStorage[slot + 277].length;
        totalSize += largeStorage[slot + 278].length;
        totalSize += largeStorage[slot + 279].length;
        totalSize += largeStorage[slot + 280].length;
        totalSize += largeStorage[slot + 281].length;
        totalSize += largeStorage[slot + 282].length;
        totalSize += largeStorage[slot + 283].length;
        totalSize += largeStorage[slot + 284].length;
        totalSize += largeStorage[slot + 285].length;
        totalSize += largeStorage[slot + 286].length;
        totalSize += largeStorage[slot + 287].length;
        totalSize += largeStorage[slot + 288].length;
        totalSize += largeStorage[slot + 289].length;
        totalSize += largeStorage[slot + 290].length;
        totalSize += largeStorage[slot + 291].length;
        totalSize += largeStorage[slot + 292].length;
        totalSize += largeStorage[slot + 293].length;
        totalSize += largeStorage[slot + 294].length;
        totalSize += largeStorage[slot + 295].length;
        totalSize += largeStorage[slot + 296].length;
        totalSize += largeStorage[slot + 297].length;
        totalSize += largeStorage[slot + 298].length;
        totalSize += largeStorage[slot + 299].length;
        totalSize += largeStorage[slot + 300].length;
        totalSize += largeStorage[slot + 301].length;
        totalSize += largeStorage[slot + 302].length;
        totalSize += largeStorage[slot + 303].length;
        totalSize += largeStorage[slot + 304].length;
        totalSize += largeStorage[slot + 305].length;
        totalSize += largeStorage[slot + 306].length;
        totalSize += largeStorage[slot + 307].length;
        totalSize += largeStorage[slot + 308].length;
        totalSize += largeStorage[slot + 309].length;
        totalSize += largeStorage[slot + 310].length;
        totalSize += largeStorage[slot + 311].length;
        totalSize += largeStorage[slot + 312].length;
        totalSize += largeStorage[slot + 313].length;
        totalSize += largeStorage[slot + 314].length;
        totalSize += largeStorage[slot + 315].length;
        totalSize += largeStorage[slot + 316].length;
        totalSize += largeStorage[slot + 317].length;
        totalSize += largeStorage[slot + 318].length;
        totalSize += largeStorage[slot + 319].length;

        // Slots 320-383
        totalSize += largeStorage[slot + 320].length;
        totalSize += largeStorage[slot + 321].length;
        totalSize += largeStorage[slot + 322].length;
        totalSize += largeStorage[slot + 323].length;
        totalSize += largeStorage[slot + 324].length;
        totalSize += largeStorage[slot + 325].length;
        totalSize += largeStorage[slot + 326].length;
        totalSize += largeStorage[slot + 327].length;
        totalSize += largeStorage[slot + 328].length;
        totalSize += largeStorage[slot + 329].length;
        totalSize += largeStorage[slot + 330].length;
        totalSize += largeStorage[slot + 331].length;
        totalSize += largeStorage[slot + 332].length;
        totalSize += largeStorage[slot + 333].length;
        totalSize += largeStorage[slot + 334].length;
        totalSize += largeStorage[slot + 335].length;
        totalSize += largeStorage[slot + 336].length;
        totalSize += largeStorage[slot + 337].length;
        totalSize += largeStorage[slot + 338].length;
        totalSize += largeStorage[slot + 339].length;
        totalSize += largeStorage[slot + 340].length;
        totalSize += largeStorage[slot + 341].length;
        totalSize += largeStorage[slot + 342].length;
        totalSize += largeStorage[slot + 343].length;
        totalSize += largeStorage[slot + 344].length;
        totalSize += largeStorage[slot + 345].length;
        totalSize += largeStorage[slot + 346].length;
        totalSize += largeStorage[slot + 347].length;
        totalSize += largeStorage[slot + 348].length;
        totalSize += largeStorage[slot + 349].length;
        totalSize += largeStorage[slot + 350].length;
        totalSize += largeStorage[slot + 351].length;
        totalSize += largeStorage[slot + 352].length;
        totalSize += largeStorage[slot + 353].length;
        totalSize += largeStorage[slot + 354].length;
        totalSize += largeStorage[slot + 355].length;
        totalSize += largeStorage[slot + 356].length;
        totalSize += largeStorage[slot + 357].length;
        totalSize += largeStorage[slot + 358].length;
        totalSize += largeStorage[slot + 359].length;
        totalSize += largeStorage[slot + 360].length;
        totalSize += largeStorage[slot + 361].length;
        totalSize += largeStorage[slot + 362].length;
        totalSize += largeStorage[slot + 363].length;
        totalSize += largeStorage[slot + 364].length;
        totalSize += largeStorage[slot + 365].length;
        totalSize += largeStorage[slot + 366].length;
        totalSize += largeStorage[slot + 367].length;
        totalSize += largeStorage[slot + 368].length;
        totalSize += largeStorage[slot + 369].length;
        totalSize += largeStorage[slot + 370].length;
        totalSize += largeStorage[slot + 371].length;
        totalSize += largeStorage[slot + 372].length;
        totalSize += largeStorage[slot + 373].length;
        totalSize += largeStorage[slot + 374].length;
        totalSize += largeStorage[slot + 375].length;
        totalSize += largeStorage[slot + 376].length;
        totalSize += largeStorage[slot + 377].length;
        totalSize += largeStorage[slot + 378].length;
        totalSize += largeStorage[slot + 379].length;
        totalSize += largeStorage[slot + 380].length;
        totalSize += largeStorage[slot + 381].length;
        totalSize += largeStorage[slot + 382].length;
        totalSize += largeStorage[slot + 383].length;
    }

    // Read multiple storage slots in one transaction
    function readStorageBatchBy384(
        uint256 startSlot,
        uint256 count
    ) public view returns (uint256) {
        uint256 totalSize = 0;
        for (uint256 i = 0; i <= count - 384; i += 384) {
            totalSize += read384Slots(startSlot + i);
        }
        return totalSize;
    }
}
