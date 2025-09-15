interface IDetailsProps {
  name: string;
  title: string;
  description: string;
  attributes: { key: string; value: string }[];
  setAttributes: (attributes: { key: string; value: string }[]) => void;
  onChangeName: (name: string) => void;
  onChangeTitle: (title: string) => void;
  onChangeDescription: (description: string) => void;
  onNext: () => void;
  onBack: () => void;
}

export const Details = ({
  name,
  title,
  description,
  attributes,
  setAttributes,
  onChangeName,
  onChangeTitle,
  onChangeDescription,
  onNext,
  onBack,
}: IDetailsProps) => {
  const handleAddAttribute = () => {
    if (attributes.length < 10) {
      setAttributes([...attributes, { key: '', value: '' }]);
    }
  };

  const handleRemoveAttribute = (index: number) => {
    const newProperties = [...attributes];
    newProperties.splice(index, 1);
    setAttributes(newProperties);
  };

  return (
    <div className="container small">
      <div className="fn_cs_contact_form">
        <ul>
          <li>
            <input
              type="text"
              placeholder="Name *"
              value={name}
              onChange={(e) => onChangeName(e.target.value)}
            />
            <input
              type="text"
              placeholder="Description *"
              style={{ marginTop: '24px' }}
              value={description}
              onChange={(e) => onChangeDescription(e.target.value)}
            />
            <input
              type="text"
              placeholder="Title (Displayed before Unlock) *"
              style={{ marginTop: '24px' }}
              value={title}
              onChange={(e) => onChangeTitle(e.target.value)}
            />
          </li>
          <li
            className="unlock_time_reciepients"
            style={{ maxHeight: '400px' }}
          >
            <button
              className="metaportal_fn_button full cursor"
              style={{ border: 'none', marginBottom: 24, zIndex: 1 }}
              onClick={handleAddAttribute}
            >
              <span>Add More Attributes (key-value)</span>
            </button>
            {attributes.map((property, index) => (
              <div key={index} style={{ display: 'flex' }}>
                <ul>
                  <li style={{ marginBottom: 24 }}>
                    <input
                      id={`property-key-${index}`}
                      type="text"
                      placeholder="Key *"
                      value={property.key}
                      onChange={(e) => {
                        const newProperties = [...attributes];
                        newProperties[index].key = e.target.value;
                        setAttributes(newProperties);
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
                        const newProperties = [...attributes];
                        newProperties[index].value = e.target.value;
                        setAttributes(newProperties);
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
                  onClick={() => handleRemoveAttribute(index)}
                >
                  -
                </div>
              </div>
            ))}
          </li>
        </ul>

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
                zIndex: 1,
                marginBottom: 24,
              }}
              onClick={onNext}
              disabled={
                attributes.some((p) => p.key === '' || p.value === '') ||
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
