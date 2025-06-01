import { useState } from 'react';

interface IDetailsProps {
  onNext: () => void;
  onBack: () => void;
}

export const Details = ({ onNext, onBack }: IDetailsProps) => {
  const [properties, setProperties] = useState<
    { key: string; value: string }[]
  >([]);

  const [title, setTitle] = useState<string>('');
  const [description, setDescription] = useState<string>('');
  const [name, setName] = useState<string>('');

  const handleAddProperty = () => {
    setProperties([...properties, { key: '', value: '' }]);
  };

  const handleRemoveRecipient = (index: number) => {
    const newProperties = [...properties];
    newProperties.splice(index, 1);
    setProperties(newProperties);
  };

  return (
    <div className="container small">
      <div className="fn_cs_contact_form">
        <ul>
          <li>
            <input
              type="text"
              placeholder="Name *"
              onChange={(e) => setName(e.target.value)}
            />
            <input
              type="text"
              placeholder="Description *"
              style={{ marginTop: '24px' }}
              onChange={(e) => setDescription(e.target.value)}
            />
            <input
              type="text"
              placeholder="Title (Displayed before Unlock) *"
              style={{ marginTop: '24px' }}
              onChange={(e) => setTitle(e.target.value)}
            />
          </li>
          <li
            className="unlock_time_reciepients"
            style={{ maxHeight: '400px' }}
          >
            <button
              className="metaportal_fn_button full cursor"
              style={{ border: 'none', marginBottom: 24, zIndex: 1 }}
              onClick={handleAddProperty}
            >
              <span>Add More Properties (key-value)</span>
            </button>
            {properties.map((property, index) => (
              <div key={index} style={{ display: 'flex' }}>
                <ul>
                  <li style={{ marginBottom: 24 }}>
                    <input
                      id={`property-key-${index}`}
                      type="text"
                      placeholder="Key *"
                      value={property.key}
                      onChange={(e) => {
                        const newProperties = [...properties];
                        newProperties[index].key = e.target.value;
                        setProperties(newProperties);
                      }}
                    />
                  </li>
                  <li style={{ marginBottom: 24 }}>
                    <input
                      id={`property-value-${index}`}
                      type="text"
                      placeholder="Value *"
                      value={property.value}
                      onChange={(e) => {
                        const newProperties = [...properties];
                        newProperties[index].value = e.target.value;
                        setProperties(newProperties);
                      }}
                    />
                  </li>
                </ul>
                <div
                  className="metaportal_fn_button full"
                  style={{
                    border: 'none',
                    zIndex: 1,
                    width: '15%',
                    fontSize: '24px',
                    cursor: 'pointer',
                    marginLeft: 24,
                  }}
                  onClick={() => handleRemoveRecipient(index)}
                >
                  -
                </div>
              </div>
            ))}
          </li>
        </ul>

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
              style={{
                border: 'none',
                zIndex: 1,
                marginBottom: 24,
              }}
              onClick={onNext}
              disabled={
                properties.some((p) => p.key === '' || p.value === '') ||
                !name ||
                !description ||
                !title
              }
            >
              <span>Next</span>
            </button>
          </li>
        </ul>
      </div>
    </div>
  );
};
