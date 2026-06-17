FROM node:20-bookworm

WORKDIR /workspace

COPY apps/web/package*.json ./apps/web/
RUN npm --prefix apps/web install

COPY . .

EXPOSE 5173

CMD ["npm", "--prefix", "apps/web", "run", "dev", "--", "--host", "0.0.0.0"]
